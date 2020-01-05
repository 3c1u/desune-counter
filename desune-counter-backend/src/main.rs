/*
 * desune-counter-backend
 * Copyright (c) 2020 MIS.W. All Rights reserverd.
 *
 * Authors: Hikaru Terazono (3c1u).
 */

#![warn(clippy::all)]

use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, http::StatusCode};
use serde::{Serialize, Deserialize};

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
        count: data.count.get()
    })
}

#[get("/api/increment")]
async fn increment(data: web::Data<CounterApp>) -> impl Responder {
    data.count.set(data.count.get() + 1);

    web::Json(CounterResponse {
        count: data.count.get()
    })
}

#[get("/desune-counter-frontend.js")]
async fn counter_js() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
                .content_type("text/javascript")
                .body(include_str!("../../target/wasm32-unknown-unknown/release/desune-counter-frontend.js"))
}

#[get("/desune-counter-frontend.wasm")]
async fn counter_wasm() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
                .content_type("application/wasm")
                .body(&include_bytes!("../../target/wasm32-unknown-unknown/release/desune-counter-frontend.wasm")[..])
}

async fn index() -> impl Responder {
    HttpResponse::build(StatusCode::OK)
                .content_type("text/html")
                .body(include_str!("../../desune-counter-frontend/static/index.html"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(||
        App::new().data(CounterApp {count: Cell::new(0)})
                  .service(count)
                  .service(increment)
                  .service(counter_js)
                  .service(counter_wasm)
                  .default_service(web::to(index))
    )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
