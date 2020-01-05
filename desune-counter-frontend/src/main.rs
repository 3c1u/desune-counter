/*
 * desune-counter-frontend
 * Copyright (c) 2020 MIS.W. All Rights reserverd.
 *
 * Authors: Hikaru Terazono (3c1u).
 */

#![warn(clippy::all)]

use yew::format::{Json, Nothing};
use yew::services::fetch::{Request, Response};
use yew::services::{FetchService, IntervalService, Task};
use yew::{html, Component, ComponentLink, Html, ShouldRender};

use serde::{Deserialize, Serialize};

use std::time::Duration;

struct Model {
    link: ComponentLink<Self>,
    count: u64,
    not_ready: bool,
    fetch: FetchService,
    _interval: IntervalService,
    _fetcher: Box<dyn Task>,
    fetch_reqeust: Box<dyn Task>,
}

#[derive(Clone, Serialize, Deserialize)]
struct CountResponse {
    count: u64,
}

enum Msg {
    Increment,
    Fetch,
    Update(CountResponse),
    DoNothing,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let mut interval = IntervalService::new();
        let task = interval.spawn(Duration::from_secs(2), link.callback(|_| Msg::Fetch));

        let mut fetch = FetchService::new();
        let fetch_reqeust = fetch.fetch(
            Request::get("/api/count").body(Nothing).unwrap(),
            link.callback(
                |response: Response<Json<Result<CountResponse, failure::Error>>>| {
                    if let Json(Ok(response)) = response.into_body() {
                        Msg::Update(response)
                    } else {
                        Msg::DoNothing
                    }
                },
            ),
        );

        Model {
            link,
            count: 0,
            not_ready: true,
            fetch,
            _interval: interval,
            _fetcher: Box::new(task),
            fetch_reqeust: Box::new(fetch_reqeust),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Increment => {
                self.count += 1;
                self.not_ready = true;

                // ですねぇリクエストをサーバーに送りつける
                let fetch = self.fetch.fetch(
                    Request::get("/api/increment").body(Nothing).unwrap(),
                    self.link.callback(
                        |response: Response<Json<Result<CountResponse, failure::Error>>>| {
                            if let Json(Ok(response)) = response.into_body() {
                                Msg::Update(response)
                            } else {
                                Msg::DoNothing
                            }
                        },
                    ),
                );

                self.fetch_reqeust = Box::new(fetch);

                true
            }
            Msg::Update(response) => {
                self.not_ready = false;
                self.count = response.count;

                true
            }
            Msg::Fetch => {
                let fetch = self.fetch.fetch(
                    Request::get("/api/count").body(Nothing).unwrap(),
                    self.link.callback(
                        |response: Response<Json<Result<CountResponse, failure::Error>>>| {
                            if let Json(Ok(response)) = response.into_body() {
                                Msg::Update(response)
                            } else {
                                Msg::DoNothing
                            }
                        },
                    ),
                );

                self.fetch_reqeust = Box::new(fetch);

                true
            }
            Msg::DoNothing => false,
        }
    }

    fn view(&self) -> Html {
        let onclick = self.link.callback(|_| Msg::Increment);
        html! {
            <div class="m-5 d-flex justify-content-center">
                <div>
                    <h1>
                        {"です"}
                        <sub>{"ねぇ"}</sub>
                        {"カウンター"}
                    </h1>
                    <p>
                    {format!("{} ですねぇ", self.count)}
                    </p>
                    <button class="btn btn-primary" onclick=onclick disabled={self.not_ready}>{ "ですねぇ" }</button>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
