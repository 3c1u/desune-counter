/*
 * desune-counter-backend
 * Copyright (c) 2020 MIS.W. All Rights reserverd.
 *
 * Authors: Hikaru Terazono (3c1u).
 */

#![warn(clippy::all)]

use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

use std::cell::Cell;

struct CounterApp {
    pub(crate) count: Cell<u64>,
}

#[derive(Serialize, Deserialize)]
struct CounterResponse {
    pub(crate) count: u64,
}

#[get("/api/count")]
async fn count(data: web::Data<CounterApp>) -> impl Responder {
    web::Json(CounterResponse {
        count: data.count.get(),
    })
}

#[get("/api/increment")]
async fn increment(data: web::Data<CounterApp>) -> impl Responder {
    data.count.set(data.count.get() + 1);

    web::Json(CounterResponse {
        count: data.count.get(),
    })
}

#[get("/desune-counter-frontend.js")]
async fn counter_js() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/javascript")
        .body(include_str!("../../static/desune-counter-frontend.js"))
}

#[get("/desune-counter-frontend.wasm")]
async fn counter_wasm() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("application/wasm")
        .body(&include_bytes!("../../static/desune-counter-frontend.wasm")[..])
}

async fn index() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html")
        .body(include_str!("../../static/index.html"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .data(CounterApp {
                count: Cell::new(0),
            })
            .service(count)
            .service(increment)
            .service(counter_js)
            .service(counter_wasm)
            .default_service(web::to(index))
    })
    .bind(format!(
        "0.0.0.0:{}",
        std::env::var("PORT") // herokuが提供するポート番号
            .unwrap_or_else(|_| "8080".into())
    ))?
    .run()
    .await
}
