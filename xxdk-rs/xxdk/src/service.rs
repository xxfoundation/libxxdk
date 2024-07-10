use std::borrow::Cow;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use base64::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json as json;
use tower::Service;

use crate::{base, rpc};

pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

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

// TODO If we're a bit more careful about it, we can probably get rid of the Sync bound here
pub trait Handler<T, S, Res>: Clone + Send + Sync + Sized + 'static {
    fn call(self, req: IncomingRequest, state: S) -> PinnedFuture<Result<Vec<u8>, String>>;
}

macro_rules! impl_handler {
    ($($ty:ident),*) => {
        impl<F, Fut, S, Res, $($ty),*> Handler<($($ty,)*), S, Res> for F
        where
            F: FnOnce($($ty),*) -> Fut + Clone + Send + Sync + Sized + 'static,
            Fut: Future<Output = Res> + Send + 'static,
            S: Send + 'static,
            Res: IntoResponse,
            $(
                $ty: FromRequest<S>,
            )*
        {
            #[allow(non_snake_case, unused_variables)]
            fn call(self, req: IncomingRequest, state: S) -> PinnedFuture<Result<Vec<u8>, String>> {
                Box::pin(async move {
                    $(
                        let $ty = $ty::extract(&req, &state)?;
                    )*
                    self($($ty),*).await.into_response()
                })
            }
        }
    };
}

macro_rules! tuples {
    ($name:ident) => {
        $name!();
        $name!(T1);
        $name!(T1, T2);
        $name!(T1, T2, T3);
        $name!(T1, T2, T3, T4);
        $name!(T1, T2, T3, T4, T5);
        $name!(T1, T2, T3, T4, T5, T6);
        $name!(T1, T2, T3, T4, T5, T6, T7);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
        $name!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
    };
}

tuples!(impl_handler);

trait ErasedHandler<S>: Send + Sync + 'static {
    fn call(&self, req: IncomingRequest, state: S) -> PinnedFuture<Result<Vec<u8>, String>>;
}

struct MakeErasedHandler<H, S> {
    handler: H,
    #[allow(clippy::type_complexity)]
    call: fn(H, IncomingRequest, S) -> PinnedFuture<Result<Vec<u8>, String>>,
}

impl<H, S> ErasedHandler<S> for MakeErasedHandler<H, S>
where
    H: Clone + Send + Sync + 'static,
    S: 'static,
{
    fn call(&self, req: IncomingRequest, state: S) -> PinnedFuture<Result<Vec<u8>, String>> {
        let h = self.handler.clone();
        (self.call)(h, req, state)
    }
}

impl<H, S> MakeErasedHandler<H, S> {
    fn make<T, Res>(handler: H) -> Self
    where
        H: Handler<T, S, Res>,
    {
        let call = |h: H, req, state| h.call(req, state);
        Self { handler, call }
    }
}

type BoxedErasedHandler<S> = Arc<dyn ErasedHandler<S>>;

#[derive(Clone)]
pub struct Router<S> {
    inner: Arc<RouterInner<S>>,
}

#[derive(Clone)]
struct RouterInner<S> {
    handlers: HashMap<String, BoxedErasedHandler<S>>,
    state: S,
}

impl Router<()> {
    pub fn without_state() -> Self {
        Self::with_state(())
    }
}

impl<S> Router<S>
where
    S: Send + Clone + 'static,
{
    pub fn with_state(state: S) -> Self {
        let handlers = HashMap::new();
        let inner = Arc::new(RouterInner { handlers, state });
        Self { inner }
    }

    pub fn route<H, T, Res>(self, endpoint: &str, handler: H) -> Self
    where
        H: Handler<T, S, Res>,
    {
        let handler = Arc::new(MakeErasedHandler::make(handler));
        self.with_inner(|inner| {
            inner.handlers.insert(String::from(endpoint), handler);
        })
    }

    fn with_inner<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut RouterInner<S>),
    {
        let mut inner = self.into_inner();
        f(&mut inner);
        Self {
            inner: Arc::new(inner),
        }
    }

    fn into_inner(self) -> RouterInner<S> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner,
            Err(arc) => RouterInner::clone(&*arc),
        }
    }
}

impl<S> Service<IncomingRequest> for Router<S>
where
    S: Clone + Send + 'static,
{
    type Response = Vec<u8>;

    type Error = String;

    type Future = PinnedFuture<Result<Vec<u8>, String>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: IncomingRequest) -> Self::Future {
        let endpoint = req.endpoint();
        let handler = match self.inner.handlers.get(req.endpoint()) {
            Some(h) => h,
            None => {
                return Box::pin(std::future::ready(Err(format!(
                    "unrecognized endpoint `{endpoint}`"
                ))))
            }
        };

        let state = self.inner.state.clone();
        handler.call(req, state)
    }
}

// TODO We can put a lifetime parameter on this to allow borrowing directly from the request buffer
pub trait FromRequest<S>: Sized {
    fn extract(req: &IncomingRequest, state: &S) -> Result<Self, String>;
}

pub trait IntoResponse {
    fn into_response(self) -> Result<Vec<u8>, String>;
}

impl<R> IntoResponse for Result<R, String>
where
    R: IntoResponse,
{
    fn into_response(self) -> Result<Vec<u8>, String> {
        self.and_then(|r| r.into_response())
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(self)
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(Vec::from(self))
    }
}

impl<const N: usize> IntoResponse for [u8; N] {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(Vec::from(&self))
    }
}

impl IntoResponse for Cow<'_, [u8]> {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(self.into_owned())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(self.into_bytes())
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(Vec::from(self.as_bytes()))
    }
}

impl IntoResponse for Cow<'_, str> {
    fn into_response(self) -> Result<Vec<u8>, String> {
        Ok(self.into_owned().into_bytes())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
{
    fn extract(req: &IncomingRequest, _state: &S) -> Result<Self, String> {
        Ok(Self(
            json::from_slice(req.request()).map_err(|e| e.to_string())?,
        ))
    }
}

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Result<Vec<u8>, String> {
        json::to_vec(&self.0).map_err(|e| e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SenderId(pub Vec<u8>);

impl<S> FromRequest<S> for SenderId {
    fn extract(req: &IncomingRequest, _state: &S) -> Result<Self, String> {
        Ok(Self(req.sender_id.clone()))
    }
}

#[derive(Debug, Clone)]
pub struct RawRequest(pub Vec<u8>);

impl<S> FromRequest<S> for RawRequest {
    fn extract(req: &IncomingRequest, _state: &S) -> Result<Self, String> {
        Ok(Self(Vec::from(req.request())))
    }
}

#[derive(Debug, Clone)]
pub struct Utf8(pub String);

impl<S> FromRequest<S> for Utf8 {
    fn extract(req: &IncomingRequest, _state: &S) -> Result<Self, String> {
        Ok(Self(String::from(
            std::str::from_utf8(req.request()).map_err(|e| e.to_string())?,
        )))
    }
}

#[derive(Debug, Clone)]
pub struct Utf8Lossy(pub String);

impl<S> FromRequest<S> for Utf8Lossy {
    fn extract(req: &IncomingRequest, _state: &S) -> Result<Self, String> {
        Ok(Self(String::from_utf8_lossy(req.request()).into_owned()))
    }
}

#[derive(Debug, Clone)]
pub struct State<S>(pub S);

impl<S> FromRequest<S> for State<S>
where
    S: Clone,
{
    fn extract(_req: &IncomingRequest, state: &S) -> Result<Self, String> {
        Ok(Self(state.clone()))
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

pub async fn serve<S>(service: S, config: CMixServerConfig) -> Result<(), String>
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
    base::callbacks::set_rpc_callbacks();
    let rpc_server = rpc::new_server(&cmix, cbs, reception_id, private_key)?;
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

impl<S> rpc::ServerCallback for CMixServerCallback<S>
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
