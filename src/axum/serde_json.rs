use crate::errors::JsonError;
use crate::{DeserializeError, Deserr};
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use std::marker::PhantomData;

/// Extractor for typed data from Json request payloads
/// deserialised by deserr.
///
/// ## Extractor
/// To extract typed data from a request body, the inner type `T` must implement the
/// [`deserr::Deserr<E>`] trait. The inner type `E` must implement the
/// [`DeserializeError`] trait.
///
/// ## Response
/// [`axum::response::IntoResponse`] is implemented for any `AxumJson<T, E>`
/// where `T` implement [`serde::Serialize`].
#[derive(Debug)]
pub struct AxumJson<T, E>(pub T, PhantomData<*const E>);

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

impl<T, S, E: DeserializeError> FromRequest<S> for AxumJson<T, E>
where
    T: Deserr<E>,
    E: DeserializeError + IntoResponse + 'static,
    S: Send + Sync,
{
    type Rejection = AxumJsonRejection<E>;

    #[inline]
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
