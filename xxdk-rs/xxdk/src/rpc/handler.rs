//! Endpoint handlers for the RPC [`Router`].

use super::*;

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

pub(crate) trait ErasedHandler<S>: Send + Sync + 'static {
    fn call(&self, req: IncomingRequest, state: S) -> PinnedFuture<Result<Vec<u8>, String>>;
}

pub(crate) struct MakeErasedHandler<H, S> {
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
    pub(crate) fn make<T, Res>(handler: H) -> Self
    where
        H: Handler<T, S, Res>,
    {
        let call = |h: H, req, state| h.call(req, state);
        Self { handler, call }
    }
}

pub(crate) type BoxedErasedHandler<S> = Arc<dyn ErasedHandler<S>>;

// TODO We can put a lifetime parameter on this to allow borrowing directly from the request buffer
pub trait FromRequest<S>: Sized {
    fn extract(req: &IncomingRequest, state: &S) -> Result<Self, String>;
}

pub trait IntoResponse {
    fn into_response(self) -> Result<Vec<u8>, String>;
}
