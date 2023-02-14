//! A module to parse query parameter as String with deserr

use std::marker::PhantomData;
use std::{fmt, ops};

use crate::{DeserializeError, Deserr};
use actix_http::Payload;
use actix_utils::future::{err, ok, Ready};
use actix_web::web::Query;
use actix_web::{FromRequest, HttpRequest, ResponseError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AwebQueryParameter<T, E>(pub T, PhantomData<*const E>);

impl<T, E> AwebQueryParameter<T, E> {
    /// Unwrap into inner `T` value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T, E> AwebQueryParameter<T, E>
where
    T: Deserr<E>,
    E: DeserializeError + ResponseError + 'static,
{
    pub fn from_query(query_str: &str) -> Result<Self, actix_web::Error> {
        let value = Query::<serde_json::Value>::from_query(query_str)?;

        match deserr::deserialize::<_, _, E>(value.0) {
            Ok(data) => Ok(AwebQueryParameter(data, PhantomData)),
            Err(e) => Err(e)?,
        }
    }
}

impl<T, E> ops::Deref for AwebQueryParameter<T, E> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, E> ops::DerefMut for AwebQueryParameter<T, E> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: fmt::Display, E> fmt::Display for AwebQueryParameter<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T, E> FromRequest for AwebQueryParameter<T, E>
where
    T: Deserr<E>,
    E: DeserializeError + ResponseError + 'static,
{
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        AwebQueryParameter::from_query(req.query_string())
            .map(ok)
            .unwrap_or_else(err)
    }
}
