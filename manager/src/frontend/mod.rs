/*
 *  YAMBA manager
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use crate::backend::Backend;
use crate::instance::Instances;
use actix::System;
use actix_web::{
    error::Result,
    fs, http,
    middleware::{self, Middleware, Started},
    server, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse, Json,
};
use failure::Fallible;
use futures::{sync::mpsc, Future, Stream};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::thread;
use tokio::{
    executor::{DefaultExecutor, Executor, SpawnError},
    runtime::Runtime,
};

mod api;
mod form;

#[derive(Clone)]
pub struct FrState {
    pub instances: Instances,
    pub backend: Backend,
}

#[derive(Fail, Debug)]
pub enum ServerErr {
    #[fail(display = "Failed to bind callback server {}", _0)]
    BindFailed(#[cause] std::io::Error),
}

/// Guard that automatically shuts down the server on drop
pub struct ShutdownGuard {
    sender: mpsc::Sender<usize>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        let _ = self.sender.try_send(1);
    }
}

pub fn init_frontend_server(
    instances: &Instances,
    backend: &Backend,
    bind_addr: SocketAddr,
) -> Fallible<ShutdownGuard> {
    let (tx, rx) = mpsc::channel(1);
    let state = FrState {
        instances: instances.clone(),
        backend: backend.clone(),
    };
    thread::spawn(move || {
        let mut sys = System::new("frontend_server");
        server::new(move || {
            App::with_state(state.clone())
                .middleware(middleware::Logger::new("manager::api::frontend"))
                .resource("/form/create/ts", |r| {
                    r.method(http::Method::POST)
                        .with_async(api::handle_create_ts)
                })
                .handler("/static", fs::StaticFiles::new("./static").unwrap())
                .handler(
                    "/",
                    fs::StaticFiles::new("./templates")
                        .unwrap()
                        .index_file("index.html"),
                )
                .boxed()
        })
        .bind(bind_addr)
        .map_err(|e| ServerErr::BindFailed(e))
        .unwrap()
        .shutdown_timeout(1)
        .run();

        sys.block_on(rx.into_future().map(|_| println!("received shutdown")))
            .unwrap();
    });

    Ok(ShutdownGuard { sender: tx })
}
