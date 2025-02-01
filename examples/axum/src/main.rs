use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::Json;
use axum::Router;
use deserr::axum::AxumJson;
use deserr::take_cf_content;
use deserr::DeserializeError;
use deserr::Deserr;
use deserr::ErrorKind;
use deserr::JsonError;
use deserr::ValuePointerRef;
use serde::Deserialize;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize, Deserialize, Deserr)]
#[serde(deny_unknown_fields)]
#[deserr(deny_unknown_fields)]
struct Query {
    name: String,

    // deserr don't do anything strange with `Option`, if you don't
    // want to make the `Option` mandatory specify it.
    #[deserr(default)]
    number: Option<i32>,

    // you can put expression in the default values
    #[serde(default = "default_range")]
    #[deserr(default = Range { min: 2, max: 4 })]
    range: Range,

    // serde support a wide variety of enums, but deserr only support
    // tagged enums, or unit enum as value.
    #[serde(rename = "return")]
    #[deserr(rename = "return")]
    returns: Return,
}

fn default_range() -> Range {
    Range { min: 2, max: 4 }
}

#[derive(Debug, Serialize, Deserialize, Deserr)]
#[serde(deny_unknown_fields)]
#[deserr(deny_unknown_fields, validate = validate_range -> __Deserr_E)]
struct Range {
    min: u8,
    max: u8,
}

// Here we could specify the error type we're going to return or stay entirely generic so the
// final caller can decide which implementation of error handler will generate the error message.
fn validate_range<E: DeserializeError>(
    range: Range,
    location: ValuePointerRef,
) -> Result<Range, E> {
    if range.min > range.max {
        Err(take_cf_content(E::error::<Infallible>(
            None,
            ErrorKind::Unexpected {
                msg: format!(
                    "`max` (`{}`) should be greater than `min` (`{}`)",
                    range.max, range.min
                ),
            },
            location,
        )))
    } else {
        Ok(range)
    }
}

#[derive(Debug, Serialize, Deserialize, Deserr)]
#[serde(rename_all = "camelCase")]
#[deserr(rename_all = camelCase)]
enum Return {
    Name,
    Number,
}

/// This handler uses the official `axum::Json` extractor
async fn serde(Json(item): Json<Query>) -> Result<Json<Query>, impl IntoResponse> {
    if item.range.min > item.range.max {
        Err((
            StatusCode::BAD_REQUEST,
            format!(
                "`max` (`{}`) should be greater than `min` (`{}`)",
                item.range.max, item.range.min
            ),
        )
            .into_response())
    } else {
        Ok(Json(item))
    }
}

/// This handler uses the official `AxumJson` deserr
async fn deserr(item: AxumJson<Query, JsonError>) -> AxumJson<Query, JsonError> {
    item
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/serde", post(serde))
        .route("/deserr", post(deserr));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

