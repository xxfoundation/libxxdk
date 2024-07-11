//! High-level `tower`-based API for cMix RPC servers.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json as json;
use tower::Service;

use crate::base;
use crate::util::PinnedFuture;

pub mod extractor;
pub mod handler;
pub mod router;

#[doc(inline)]
pub use router::Router;

#[derive(Debug, Clone)]
pub struct IncomingRequest {
    sender_id: Vec<u8>,
    request: Vec<u8>,
    separator_idx: usize,
}

impl IncomingRequest {
    fn new(sender_id: Vec<u8>, request: Vec<u8>) -> Result<Self, String> {
        let separator_idx = request
            .iter()
            .position(|b| *b == b',')
            .ok_or_else(|| "no endpoint in request".to_string())?;

        std::str::from_utf8(&request[..separator_idx])
            .map_err(|e| format!("non-UTF-8 endpoint: {e}"))?;

        Ok(Self {
            sender_id,
            request,
            separator_idx,
        })
    }

    pub fn sender_id(&self) -> &[u8] {
        &self.sender_id
    }

    pub fn endpoint(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.request[..self.separator_idx]) }
    }

    pub fn request(&self) -> &[u8] {
        &self.request[self.separator_idx + 1..]
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcServerConfig {
    pub ndf_path: String,
    pub storage_dir: String,
    pub secret: String,
    pub reception_id: String,
    pub private_key: String,
}

pub async fn serve<S>(service: S, config: RpcServerConfig) -> Result<(), String>
where
    S: Service<IncomingRequest, Response = Vec<u8>, Error = String> + Clone + Send + 'static,
{
    tracing::info!("Starting cMix server");
    let ndf_contents = tokio::fs::read_to_string(&config.ndf_path)
        .await
        .map_err(|e| e.to_string())?;

    if tokio::fs::read_dir(&config.storage_dir).await.is_err() {
        let storage_dir = config.storage_dir.clone();
        let secret = config.secret.clone();
        tokio::task::spawn_blocking(move || {
            tracing::info!("Creating storage directory");
            base::CMix::create(&ndf_contents, &storage_dir, secret.as_bytes(), "")
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    let storage_dir = config.storage_dir.clone();
    let secret = config.secret.clone();
    let cmix = tokio::task::spawn_blocking(move || {
        tracing::info!("Loading storage directory");
        base::CMix::load(&storage_dir, secret.as_bytes(), &[])
    })
    .await
    .map_err(|e| e.to_string())??;

    // Reception ID Load or Generate
    let mut reception_id_b64 = config.reception_id.clone();
    let reception_id: Vec<u8>;
    if reception_id_b64.is_empty() {
        match cmix.ekv_get("rpc_server_reception_id") {
            Ok(r) => {
                tracing::info!("Loaded Reception ID From EKV...");
                reception_id = r;
            }
            Err(_) => {
                tracing::info!("Generating Random Reception ID...");
                reception_id = base::rpc::generate_reception_id(&cmix)?;
            }
        }
    } else {
        match BASE64_STANDARD_NO_PAD.decode(reception_id_b64) {
            Ok(r) => {
                tracing::info!("Loaded Reception ID From config...");
                reception_id = r;
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
    reception_id_b64 = BASE64_STANDARD_NO_PAD.encode(&reception_id);
    tracing::info!("RPC Reception ID: {reception_id_b64}");
    cmix.ekv_set("rpc_server_reception_id", &reception_id)?;

    // Private Key Load or Generate
    let private_key_b64 = config.private_key.clone();
    let private_key: Vec<u8>;
    if private_key_b64.is_empty() {
        match cmix.ekv_get("rpc_server_private_key") {
            Ok(r) => {
                private_key = r;
                tracing::info!("Loaded Private Key From EKV...");
            }
            Err(_) => {
                private_key = base::rpc::generate_random_key(&cmix)?;
                tracing::info!("Generating Random Private Key...");
            }
        }
    } else {
        match BASE64_STANDARD_NO_PAD.decode(private_key_b64) {
            Ok(r) => {
                tracing::info!("Loaded Private Key From config...");
                private_key = r;
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
    let public_key = base::rpc::derive_public_key(&private_key)?;
    let public_key_b64 = BASE64_STANDARD_NO_PAD.encode(public_key);
    tracing::info!("RPC Public Key: {public_key_b64}");
    cmix.ekv_set("rpc_server_private_key", &private_key)?;

    let runtime = tokio::runtime::Handle::current();
    let cbs = CMixServerCallback { service, runtime };

    let cmix = Arc::new(cmix);
    tokio::task::spawn_blocking({
        let cmix = cmix.clone();
        move || {
            tracing::info!("Starting network follower");
            cmix.start_network_follower(5000)?;
            while let Err(e) = cmix.wait_for_network(20000) {
                tracing::info!("Waiting to connect to network: {e}");
            }
            Ok::<_, String>(())
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    tracing::info!("Waiting until ready to send");
    while !cmix.ready_to_send() {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    tracing::info!("Spawning RPC server");
    base::rpc::set_rpc_callbacks();
    let rpc_server = cmix.new_rpc_server(cbs, reception_id, private_key)?;
    rpc_server.start();
    tracing::info!(
        "RPC Server CB PTR: {:#x}",
        rpc_server.cb as *const _ as *const libc::c_void as usize
    );
    tracing::info!("RPC Server Started");
    tracing::info!("RPC Public Key: {public_key_b64}");
    tracing::info!("RPC Reception ID: {reception_id_b64}");

    // TODO We need a better way to shut down the server. This never actually completes or gets
    // past this line, it just runs until the process gets a kill signal.
    std::future::pending::<()>().await;

    rpc_server.stop();
    cmix.stop_network_follower()
}

struct CMixServerCallback<S> {
    service: S,
    runtime: tokio::runtime::Handle,
}

impl<S> base::rpc::ServerCallback for CMixServerCallback<S>
where
    S: Service<IncomingRequest, Response = Vec<u8>, Error = String> + Clone + Send + 'static,
{
    fn serve_req(&self, sender_id: Vec<u8>, request: Vec<u8>) -> Vec<u8> {
        let mut service = self.service.clone();
        let res: Result<Vec<u8>, String> = self.runtime.block_on(async move {
            tracing::debug!("evaluating service on request");
            if std::future::poll_fn(|cx| service.poll_ready(cx))
                .await
                .is_ok()
            {
                let req = IncomingRequest::new(sender_id, request)?;
                service.call(req).await
            } else {
                Err("unable to service request".to_string())
            }
        });

        let res = match res {
            Ok(bytes) => bytes,
            Err(text) => {
                tracing::warn!(error = text, "error servicing request");
                text.into_bytes()
            }
        };

        tracing::info!(
            res = String::from_utf8_lossy(&res).as_ref(),
            "sending response"
        );
        res
    }
}
