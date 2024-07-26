//! `tower`-based `Service` for use with the cMix RPC API.

use super::*;

use crate::rpc::handler::*;

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
