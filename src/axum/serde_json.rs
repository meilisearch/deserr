use crate::{DeserializeError, Deserr, JsonError};
use axum::async_trait;
use axum::body::HttpBody;
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use axum::{BoxError, Json};
use http::{Request, StatusCode};
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
/// [`axum::IntoResponse`] is implemented for any `AxumJson<T, E>`
/// where `T` implement [`serde::Serialize`].
#[derive(Debug)]
pub struct AxumJson<T, E>(pub T, PhantomData<E>);

#[derive(Debug)]
pub enum AxumJsonRejection {
    DeserrError(JsonError),
    JsonRejection(JsonRejection),
}

impl<T, E> IntoResponse for AxumJson<T, E>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        Json(self.0).into_response()
    }
}

#[async_trait]
impl<T, S, B, E: DeserializeError> FromRequest<S, B> for AxumJson<T, E>
where
    T: Deserr<E>,
    E: DeserializeError + 'static,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
    AxumJsonRejection: std::convert::From<E>,
{
    type Rejection = AxumJsonRejection;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value): Json<serde_json::Value> = Json::from_request(req, state).await?;
        let data = deserr::deserialize::<_, _, _>(value)?;
        Ok(AxumJson(data, PhantomData))
    }
}

impl From<JsonError> for AxumJsonRejection {
    fn from(value: JsonError) -> Self {
        AxumJsonRejection::DeserrError(value)
    }
}

impl From<JsonRejection> for AxumJsonRejection {
    fn from(value: JsonRejection) -> Self {
        AxumJsonRejection::JsonRejection(value)
    }
}

impl IntoResponse for AxumJsonRejection {
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
