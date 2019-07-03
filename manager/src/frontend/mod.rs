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
use actix_files as fs;
use actix_web::{http, middleware, web, App, HttpServer};
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
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Logger::new("manager::api::frontend"))
            .service(
                web::resource("/api/instances/create")
                    .route(web::post().to(api::handle_instances_create)),
            )
            .service(
                web::resource("/api/instances/{instance}/core")
                    .route(web::get().to(api::handle_instance_config_core_get))
                    .route(web::put().to(api::handle_instance_config_core_update)),
            )
            .service(
                web::resource("/api/instances/{instance}/stop")
                    .route(web::post().to_async(api::handle_instance_stop)),
            )
            .service(
                web::resource("/api/instances/{instance}/start")
                    .route(web::post().to_async(api::handle_instance_start)),
            )
            .service(
                web::resource("/api/instances/{instance}")
                    .route(web::delete().to(api::handle_instance_delete)),
            )
            .service(
                web::resource("/api/playback/{instance}/volume")
                    .route(web::get().to(api::handle_volume_get)),
            )
            .service(
                web::resource("/api/playback/{instance}/state")
                    .route(web::get().to(api::handle_playback_get)),
            )
            .service(
                web::resource("/api/playback/{instance}/track")
                    .route(web::get().to(api::handle_track_get)),
            )
            .service(
                web::resource("/api/instances").route(web::get().to(api::handle_instances_get)),
            )
            .service(web::resource("/ws").to(ws::ws_route))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind(bind_addr)
    .map_err(|e| ServerErr::BindFailed(e))
    .unwrap()
    .shutdown_timeout(1)
    .start();

    Ok(())
}
