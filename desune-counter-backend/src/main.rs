/*
 * desune-counter-backend
 * Copyright (c) 2020 MIS.W. All Rights reserverd.
 *
 * Authors: Hikaru Terazono (3c1u).
 */

#![warn(clippy::all)]

use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::time::{Instant, Duration};

use std::cell::Cell;

use tokio_postgres::Client;
use std::sync::Arc;

struct CounterApp {
    pub(crate) count: Cell<u64>,
    pub(crate) last_time: Cell<Instant>,
    pub(crate) is_active: Cell<bool>,
    pub(crate) client: Arc<Client>,
}

#[derive(Serialize, Deserialize)]
struct CounterResponse {
    pub(crate) count: u64,
    pub(crate) is_active: bool,
}

#[get("/api/count")]
async fn count(data: web::Data<CounterApp>) -> impl Responder {
    if Duration::from_secs(15) <= (Instant::now() - data.last_time.get()) {
        if let Some((is_active, count_num)) = get_counter(&data.client).await {
            data.count.set(count_num);
            data.is_active.set(is_active);
            data.last_time.set(Instant::now());
        } else {
            eprintln!("database failure");
        }
    }

    web::Json(CounterResponse {
        count: data.count.get(),
        is_active: data.is_active.get(),
    })
}

#[get("/api/increment")]
async fn increment(data: web::Data<CounterApp>) -> impl Responder {
    if let Some((is_active, count_num)) = increment_desune(&data.client).await {
        data.count.set(count_num);
        data.is_active.set(is_active);
        data.last_time.set(Instant::now());
    } else {
        eprintln!("database failure");
    }

    web::Json(CounterResponse {
        count: data.count.get(),
        is_active: data.is_active.get(),
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

static DB_URL: &str = "postgres://senko:uyan@127.0.0.1:5432/senko";

async fn init_database() -> Client {
    use tokio_postgres::NoTls;

    let database_url = std::env::var("DATABASE_URL") // herokuが提供するデータベース
                        .unwrap_or_else(|_| DB_URL.into());
    let (client, conn) = tokio_postgres::connect(&database_url, NoTls).await.expect("failed to connect");

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.execute(r#"CREATE TABLE IF NOT EXISTS desune_counter (
        key INT8 PRIMARY KEY,
        time TIMESTAMP WITH TIME ZONE default CURRENT_TIMESTAMP,
        count INT8
    )"#, &[]).await.unwrap();

    client.execute(r#"INSERT INTO desune_counter (key, count) VALUES (0, 0) ON CONFLICT DO NOTHING"#, &[]).await.unwrap();

    client
}

async fn increment_desune(client: &Client) ->  Option<(bool, u64)> {
    client.execute(r#"UPDATE desune_counter SET count = count+1, time = clock_timestamp() WHERE clock_timestamp() - time >= '00:00:10'"#, &[]).await.ok()?;
    get_counter(client).await
}

async fn get_counter(client: &Client) -> Option<(bool, u64)> {
    let res = client.query(r#"SELECT clock_timestamp() - time >= '00:00:10', count FROM desune_counter"#, &[]).await.ok()?;
    
    if res.len() == 1 {
        let (flag, count_num): (bool, i64) = (res[0].get(0), res[0].get(1));
        Some((flag, count_num as u64))
    } else {
       None
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let client = Arc::new(init_database().await);

    HttpServer::new(move || {
        App::new()
            .data(CounterApp {
                count: Cell::default(),
                last_time: Cell::new(Instant::now()),
                is_active: Cell::default(),
                client: client.clone(),
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
