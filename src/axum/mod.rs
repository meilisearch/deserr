#[cfg(feature = "serde-json")]
mod serde_json;

#[cfg(feature = "serde-json")]
pub use self::serde_json::AxumJson;
