use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json as json;
use tower::Service;

#[derive(Debug, Clone, Deserialize)]
struct RawRequest {
    method: String,
    #[serde(flatten)]
    params: json::Value,
}

#[derive(Debug, Clone)]
pub struct RequestMeta {
    pub message_id: Vec<u8>,
    pub sender_key: Vec<u8>,
    pub sender_token: i32,
    pub timestamp: i64,
}

pub struct IncomingRequest {
    body: RawRequest,
    meta: RequestMeta,
}

pub type PinnedFuture<T> = Pin<Box<dyn Future<Output = T> + Send + 'static>>;

pub trait Handler<Fut, P, R, S>: Clone + Send + 'static {
    fn call(self, params: P, meta: RequestMeta, state: &S) -> Fut;
}

impl<F, Fut, P, R, S> Handler<Fut, P, R, S> for F
where
    F: FnOnce(P, RequestMeta, &S) -> Fut + Clone + Send + 'static,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
    P: DeserializeOwned,
    R: Serialize,
    S: Clone + Send + 'static,
{
    fn call(self, params: P, meta: RequestMeta, state: &S) -> Fut {
        (self)(params, meta, state)
    }
}

struct HandlerService<H, Fut, P, R, S> {
    handler: H,
    state: S,
    _phantom: PhantomData<fn(P) -> (Fut, R)>,
}

impl<H, Fut, P, R, S> Clone for HandlerService<H, Fut, P, R, S>
where
    H: Clone,
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            state: self.state.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<H, Fut, P, R, S> Service<IncomingRequest> for HandlerService<H, Fut, P, R, S>
where
    H: Handler<Fut, P, R, S>,
    Fut: Future<Output = Result<R, String>> + Send + 'static,
    P: DeserializeOwned,
    R: Serialize,
    S: Clone + Send + 'static,
{
    type Response = Vec<u8>;

    type Error = String;

    type Future = PinnedFuture<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: IncomingRequest) -> Self::Future {
        let service = self.clone();
        Box::pin(async move {
            let IncomingRequest { body, meta } = req;
            let RawRequest { params, .. } = body;
            let params = json::from_value(params).map_err(|e| e.to_string())?;

            let state = &service.state;
            let res = service.handler.call(params, meta, state).await?;

            json::to_vec(&res).map_err(|e| e.to_string())
        })
    }
}
