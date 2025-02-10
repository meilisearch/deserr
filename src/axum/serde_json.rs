use std::marker::PhantomData;

use crate::errors::JsonError;
use crate::{DeserializeError, Deserr};
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;

/// Extractor for typed data from Json request payloads
/// deserialised by deserr.
///
/// ## Extractor
/// To extract typed data from a request body, the inner type `T` must implement the
/// [`deserr::Deserr<E>`] trait. The inner type `E` must implement the
/// [`DeserializeError`] trait.
#[derive(Debug)]
pub struct AxumJson<T, E>(pub T, PhantomData<E>);

impl<T, E> AxumJson<T, E> {
    pub fn new(data: T) -> Self {
        AxumJson(data, PhantomData)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug)]
pub enum AxumJsonRejection<E: DeserializeError> {
    DeserrError(E),
    JsonRejection(JsonRejection),
}

impl<E: DeserializeError + std::fmt::Display> std::fmt::Display for AxumJsonRejection<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AxumJsonRejection::DeserrError(e) => e.fmt(f),
            AxumJsonRejection::JsonRejection(e) => e.fmt(f),
        }
    }
}

impl<T, E, S> FromRequest<S> for AxumJson<T, E>
where
    E: DeserializeError + IntoResponse + 'static,
    T: Deserr<E>,
    S: Send + Sync,
{
    type Rejection = AxumJsonRejection<E>;

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<serde_json::Value>::from_request(req, state).await?;
        let data = deserr::deserialize::<_, _, _>(value)?;
        Ok(AxumJson(data, PhantomData))
    }
}

impl<E: DeserializeError> From<E> for AxumJsonRejection<E> {
    fn from(value: E) -> Self {
        AxumJsonRejection::DeserrError(value)
    }
}

impl<E: DeserializeError> From<JsonRejection> for AxumJsonRejection<E> {
    fn from(value: JsonRejection) -> Self {
        AxumJsonRejection::JsonRejection(value)
    }
}

impl<E: DeserializeError + IntoResponse> IntoResponse for AxumJsonRejection<E> {
    fn into_response(self) -> axum::response::Response {
        match self {
            AxumJsonRejection::DeserrError(e) => e.into_response(),
            AxumJsonRejection::JsonRejection(e) => e.into_response(),
        }
    }
}

impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
