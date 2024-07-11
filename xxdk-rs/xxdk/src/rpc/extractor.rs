//! Request extractor implementations for use in RPC endpoint handlers.

use super::*;

use std::borrow::Cow;

use serde::de::DeserializeOwned;

use crate::rpc::handler::*;

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
