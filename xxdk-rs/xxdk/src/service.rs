use crate::base::callbacks;
use crate::base::{self, generate_codename_identity};

use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::{Arc, OnceLock};
use std::task::Poll;
use std::time::Duration;

use base64::prelude::*;
use serde::Deserialize;
use tokio::sync::mpsc;
use tower::Service;

const DM_ID_EKV_KEY: &str = "MyDMID";

#[derive(Debug, Clone)]
pub struct IncomingRequest {
    pub text: Vec<u8>,
    pub sender_key: Vec<u8>,
    pub dm_token: i32,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
struct Response {
    text: Vec<u8>,
    partner_pubkey: Vec<u8>,
    partner_token: i32,
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

        let dm_id = cmix.ekv_get(DM_ID_EKV_KEY).or_else(|_| {
            tracing::info!("Generating DM ID");
            let id = generate_codename_identity(&config.secret);
            cmix.ekv_set(DM_ID_EKV_KEY, &id)?;
            Ok::<_, String>(id)
        })?;

        let runtime = tokio::runtime::Handle::current();
        let (sender, mut response_queue) = mpsc::channel(256);
        let cbs_pubkey_lock = Arc::new(OnceLock::new());
        let cbs = Arc::new(CMixServerCallbacks {
            router,
            runtime,
            response_queue: sender,
            server_pubkey: cbs_pubkey_lock.clone(),
        });
        tracing::info!("Spawning DM client");
        let dm = cmix.new_dm_client(&dm_id, &config.secret, cbs)?;

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

        let token = dm.get_token()?;
        let pubkey = dm.get_dm_pubkey()?;
        let pubkey_display = BASE64_STANDARD_NO_PAD.encode(&pubkey);
        tracing::info!("DMTOKEN: {}", token as u32);
        tracing::info!("DMPUBKEY: {pubkey_display}");

        // This shouldn't block, and should return Ok, since it's the only place where this cell is
        // initialized.
        cbs_pubkey_lock.set(pubkey).unwrap();

        tracing::debug!("Listening for messages");
        while let Some(resp) = response_queue.recv().await {
            tracing::debug!("Sending response");
            for window in resp.text.chunks(750) {
                if let Err(e) = dm.send(&resp.partner_pubkey, resp.partner_token, 0, window, 0, &[])
                {
                    tracing::warn!(error = e, "Error sending response");
                    break;
                }

                tokio::time::sleep(Duration::from_secs(4)).await;
            }
        }

        cmix.stop_network_follower()
    }
}

struct CMixServerCallbacks<T> {
    router: Router<T>,
    server_pubkey: Arc<OnceLock<Vec<u8>>>,
    runtime: tokio::runtime::Handle,
    response_queue: mpsc::Sender<Response>,
}

impl<T> CMixServerCallbacks<T>
where
    T: Send + Sync + 'static,
{
    fn serve_req(&self, text: &[u8], sender_key: &[u8], dm_token: i32, timestamp: i64) {
        if sender_key != self.server_pubkey.get().unwrap() {
            let mut router = self.router.clone();
            let req = IncomingRequest {
                text: text.into(),
                sender_key: sender_key.into(),
                dm_token,
                timestamp,
            };
            let response_queue = self.response_queue.clone();
            let sender_key = Vec::from(sender_key);
            self.runtime.spawn(async move {
                tracing::debug!("Evaluating router on request");
                if poll_fn(|cx| router.poll_ready(cx)).await.is_ok() {
                    match router.call(req).await {
                        Err(e) => {
                            tracing::warn!(error = e, "Error in servicing request");
                        }
                        Ok(resp) => {
                            if response_queue
                                .send(Response {
                                    text: resp,
                                    partner_pubkey: sender_key,
                                    partner_token: dm_token,
                                })
                                .await
                                .is_err()
                            {
                                tracing::warn!(partner_token = dm_token, "Error queuing response");
                            }
                        }
                    }
                }
            });
        }
    }
}

impl<T> callbacks::DmCallbacks for CMixServerCallbacks<T>
where
    T: Send + Sync + 'static,
{
    fn receive(
        &self,
        _message_id: &[u8],
        _nickname: &str,
        text: &[u8],
        _partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        _codeset: i32,
        timestamp: i64,
        _round_id: i64,
        _message_type: i64,
        _status: i64,
    ) -> i64 {
        tracing::debug!(dm_token, "Received raw");
        self.serve_req(text, sender_key, dm_token, timestamp);
        0
    }

    fn receive_text(
        &self,
        _message_id: &[u8],
        _nickname: &str,
        text: &str,
        _partner_key: &[u8],
        sender_key: &[u8],
        dm_token: i32,
        _codeset: i32,
        timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        tracing::debug!(dm_token, "Received text");
        self.serve_req(text.as_bytes(), sender_key, dm_token, timestamp);
        0
    }

    fn receive_reply(
        &self,
        _message_id: &[u8],
        _reply_to: &[u8],
        _nickname: &str,
        _text: &str,
        _partner_key: &[u8],
        _sender_key: &[u8],
        dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        tracing::debug!(dm_token, "Received reply");
        0
    }

    fn receive_reaction(
        &self,
        _message_id: &[u8],
        _reaction_to: &[u8],
        _nickname: &str,
        _text: &str,
        _partner_key: &[u8],
        _sender_key: &[u8],
        dm_token: i32,
        _codeset: i32,
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) -> i64 {
        tracing::debug!(dm_token, "Received reaction");
        0
    }

    fn update_sent_status(
        &self,
        _uuid: i64,
        _message_id: &[u8],
        _timestamp: i64,
        _round_id: i64,
        _status: i64,
    ) {
    }

    fn block_sender(&self, _pubkey: &[u8]) {}

    fn unblock_sender(&self, _pubkey: &[u8]) {}

    fn get_conversation(&self, _pubkey: &[u8]) -> Vec<u8> {
        vec![]
    }

    fn get_conversations(&self) -> Vec<u8> {
        vec![]
    }

    fn delete_message(&self, _message_id: &[u8], _pubkey: &[u8]) -> bool {
        false
    }

    fn event_update(&self, _event_type: i64, _json_data: &[u8]) {}
}
