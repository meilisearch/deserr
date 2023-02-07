//! This module holds some pre-made error types to eases your usage of deserr

pub mod helpers;
pub mod json;
pub mod query_params;

pub use json::JsonError;
pub use query_params::QueryParamError;
