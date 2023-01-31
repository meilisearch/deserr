use std::fmt::Debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_web::dev::Payload;
use actix_web::web::Json;
use actix_web::{FromRequest, HttpRequest, ResponseError};
use deserr::{DeserializeError, Deserr};
use futures::ready;

use crate::serde_json::JsonError;

/// Extractor for typed data from Json request payloads
/// deserialised by deserr.
///
/// # Extractor
/// To extract typed data from a request body, the inner type `T` must implement the
/// [`deserr::Deserr<E>`] trait. The inner type `E` must implement the
/// [`DeserializeError`] + `ResponseError` traits.
#[derive(Debug)]
pub struct AwebJson<T, E>(pub T, PhantomData<*const E>);

impl<T, E> AwebJson<T, E> {
    pub fn new(data: T) -> Self {
        AwebJson(data, PhantomData)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, E> FromRequest for AwebJson<T, E>
where
    E: DeserializeError + ResponseError + 'static,
    T: Deserr<E>,
{
    type Error = actix_web::Error;
    type Future = AwebJsonExtractFut<T, E>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        AwebJsonExtractFut {
            fut: Json::<serde_json::Value>::from_request(req, payload),
            _phantom: PhantomData,
        }
    }
}

pub struct AwebJsonExtractFut<T, E> {
    fut: <Json<serde_json::Value> as FromRequest>::Future,
    _phantom: PhantomData<*const (T, E)>,
}

impl<T, E> Future for AwebJsonExtractFut<T, E>
where
    T: Deserr<E>,
    E: DeserializeError + ResponseError + 'static,
{
    type Output = Result<AwebJson<T, E>, actix_web::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let AwebJsonExtractFut { fut, .. } = self.get_mut();
        let fut = Pin::new(fut);

        let res = ready!(fut.poll(cx));

        let res = match res {
            Err(err) => Err(err),
            Ok(data) => match deserr::deserialize::<_, _, E>(data.into_inner()) {
                Ok(data) => Ok(AwebJson::new(data)),
                Err(e) => Err(e)?,
            },
        };

        Poll::Ready(res)
    }
}

impl actix_web::ResponseError for JsonError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponseBuilder::new(self.status_code())
            .content_type("text/plain")
            .body(self.to_string())
    }
}
