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
use actix_web::{fs, http, middleware, server, App};
use failure::Fallible;
use std::net::SocketAddr;

pub use ws::{InstanceCreated, WSServer};

mod api;
mod ws;

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

/// Init frontend server + websockets
pub fn init_frontend_server(
    instances: Instances,
    backend: Backend,
    bind_addr: SocketAddr,
) -> Fallible<()> {
    let state = FrState {
        instances: instances.clone(),
        backend: backend.clone(),
    };
    server::new(move || {
        App::with_state(state.clone())
            .middleware(middleware::Logger::new("manager::api::frontend"))
            .resource("/api/instances/{instance}/core", |r| {
                r.method(http::Method::GET)
                    .with(api::handle_instance_config_core)
            })
            .resource("/api/instances/{instance}/stop", |r| {
                r.method(http::Method::POST)
                    .with_async(api::handle_instance_stop)
            })
            .resource("/api/instances/create", |r| {
                r.method(http::Method::POST)
                    .with_async(api::handle_instances_create)
            })
            .resource("/api/playback/{instance}/volume", |r| {
                r.method(http::Method::GET).with(api::handle_volume_get)
            })
            .resource("/api/playback/{instance}/state", |r| {
                r.method(http::Method::GET).with(api::handle_playback_get)
            })
            .resource("/api/playback/{instance}/track", |r| {
                r.method(http::Method::GET).with(api::handle_track_get)
            })
            .resource("/api/instances", |r| {
                r.method(http::Method::GET).with(api::handle_instances_get)
            })
            .resource("/ws", |r| r.route().f(ws::ws_route))
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
    .start();

    Ok(())
}
