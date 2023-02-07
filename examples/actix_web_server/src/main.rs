use std::convert::Infallible;

use actix_web::{web, App, HttpResponse, HttpServer};
use deserr::{
    actix_web::AwebJson, errors::JsonError, take_cf_content, DeserializeError, Deserr, ErrorKind,
    ValuePointerRef,
};
use serde::{Deserialize, Serialize};

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

/// This handler uses the official `actix_web` `serde_json` extractor
async fn serde(item: web::Json<Query>) -> HttpResponse {
    if item.range.min > item.range.max {
        HttpResponse::BadRequest().body(format!(
            "`max` (`{}`) should be greater than `min` (`{}`)",
            item.range.max, item.range.min
        ))
    } else {
        HttpResponse::Ok().json(item.0)
    }
}

/// This handler uses the official `actix_web` `serde_json` extractor
async fn deserr(item: AwebJson<Query, JsonError>) -> HttpResponse {
    HttpResponse::Ok().json(item.0)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("starting HTTP server at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/serde").route(web::post().to(serde)))
            .service(web::resource("/deserr").route(web::post().to(deserr)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
