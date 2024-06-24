use base64::prelude::*;
use serde::Deserialize;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use tokio::sync::mpsc;
use tower::Service;

use crate::base;
use crate::rpc;

#[derive(Debug, Clone)]
pub struct IncomingRequest {
    pub sender_id: Vec<u8>,
    pub request: Vec<u8>,
}
#[derive(Debug, Clone)]
struct Response {
    pub text: Vec<u8>,
}

type HandlerFnInner<T> = dyn Fn(
        Arc<T>,
        IncomingRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + Send + 'static>>
    + Send
    + Sync
    + 'static;

pub trait HandlerFn<T>:
    Fn(Arc<T>, IncomingRequest) -> Self::Future + Send + Sync + 'static
{
    type Future: Future<Output = Result<Vec<u8>, String>> + Send + 'static;
}

impl<T, F, Fut> HandlerFn<T> for F
where
    F: Fn(Arc<T>, IncomingRequest) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<u8>, String>> + Send + 'static,
{
    type Future = Fut;
}

pub struct Router<T> {
    handler: Arc<HandlerFnInner<T>>,
    state: Arc<T>,
}

impl<T> Clone for Router<T> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            state: self.state.clone(),
        }
    }
}

impl<T> Router<T> {
    pub fn new<F>(handler: F, state: Arc<T>) -> Self
    where
        F: HandlerFn<T>,
    {
        Self {
            handler: Arc::new(move |state, req| Box::pin(handler(state, req))),
            state,
        }
    }
}

impl<T> Service<IncomingRequest> for Router<T>
where
    T: Send + Sync + 'static,
{
    type Response = Vec<u8>;

    type Error = String;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: IncomingRequest) -> Self::Future {
        (self.handler)(self.state.clone(), req)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CMixServerConfig {
    pub ndf_path: String,
    pub storage_dir: String,
    pub secret: String,
    pub reception_id: String,
    pub private_key: String,
}

#[derive(Debug)]
pub struct CMixServer;

impl CMixServer {
    pub async fn serve<T>(router: Router<T>, config: CMixServerConfig) -> Result<(), String>
    where
        T: Send + Sync + 'static,
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
                    reception_id = rpc::generate_reception_id(&cmix)?;
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
                    private_key = rpc::generate_random_key(&cmix)?;
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
        let public_key = rpc::derive_public_key(&private_key)?;
        let public_key_b64 = BASE64_STANDARD_NO_PAD.encode(public_key);
        tracing::info!("RPC Public Key: {public_key_b64}");
        cmix.ekv_set("rpc_server_private_key", &private_key)?;

        let runtime = tokio::runtime::Handle::current();
        let (sender, mut response_queue) = mpsc::channel(256);
        let cbs = CMixServerCallbacks {
            router,
            runtime,
            response_queue: sender,
        };

        tracing::info!("Spawning RPC server");
        base::callbacks::set_rpc_callbacks();
        let rpc_server = rpc::new_server(&cmix, cbs, reception_id, private_key)?;

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

        rpc_server.start();

        tracing::info!(
            "RPC Server CB PTR: {:#x}",
            rpc_server.cb as *const _ as *const libc::c_void as usize
        );
        tracing::info!("RPC Server Started");
        tracing::info!("RPC Public Key: {public_key_b64}");
        tracing::info!("RPC Reception ID: {reception_id_b64}");

        while let Some(resp) = response_queue.recv().await {
            tokio::spawn(async move {
                tracing::debug!("request received, sending response");
                tracing::debug!("{:?}", resp.text);
            });
        }

        // rpc_server.stop();
        cmix.stop_network_follower()
    }
}

unsafe impl Send for CMixServer {}

struct CMixServerCallbacks<T> {
    router: Router<T>,
    runtime: tokio::runtime::Handle,
    response_queue: mpsc::Sender<Response>,
}

impl<T> rpc::ServerCallback for CMixServerCallbacks<T>
where
    T: Send + Sync + 'static,
{
    fn serve_req(&self, sender_id: Vec<u8>, request: Vec<u8>) -> Vec<u8> {
        let mut router = self.router.clone();
        let response_queue = self.response_queue.clone();
        let req = IncomingRequest { sender_id, request };
        let r = self.runtime.block_on(async {
            let ret: Vec<u8>;
            tracing::debug!("Evaluating router on request");
            if poll_fn(|cx| router.poll_ready(cx)).await.is_ok() {
                match router.call(req).await {
                    Err(e) => {
                        tracing::warn!(error = e, "Error in servicing request");
                        ret = e.into_bytes();
                    }
                    Ok(resp) => {
                        if response_queue
                            .send(Response { text: resp.clone() })
                            .await
                            .is_err()
                        {
                            tracing::warn!("couldn't send to queue");
                        };
                        ret = resp;
                    }
                }
            } else {
                ret = String::from("error unable to service request").into_bytes();
            }
            ret
        });
        return r;
    }
}
