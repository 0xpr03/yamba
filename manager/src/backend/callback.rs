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

use super::*;
use crate::frontend;
use crate::security::SecurityModule;

use actix::SystemService;
use actix_web::{http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use failure::Fallible;
use yamba_types::models::callback as cb;

#[derive(Fail, Debug)]
pub enum ServerErr {
    #[fail(display = "Failed to bind callback server {}", _0)]
    BindFailed(#[cause] std::io::Error),
}

/// Handle instance callback
fn callback_instance(
    (data, req): (
        web::Json<cb::InstanceStateResponse>,
        web::Data<CallbackState>,
    ),
) -> HttpResponse {
    debug!("Instance state change: {:?}", data);
    if let Some(i) = req.instances.read(&data.id) {
        i.cb_set_instance_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_volume(
    (data, req): (web::Json<cb::VolumeChange>, web::Data<CallbackState>),
) -> HttpResponse {
    debug!("Volume change: {:?}", data);
    if let Some(i) = req.instances.read(&data.id) {
        i.cb_update_volume(data.into_inner().volume);
    }
    HttpResponse::Ok().json(true)
}

fn callback_playback(
    (data, req): (web::Json<cb::PlaystateResponse>, web::Data<CallbackState>),
) -> HttpResponse {
    debug!("Playback change: {:?}", data);
    if let Some(i) = req.instances.read(&data.id) {
        i.cb_set_playback_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_resolve(
    (body, req): (web::Json<cb::ResolveResponse>, web::Data<CallbackState>),
) -> HttpResponse {
    debug!("Resolve callback: {:?}", body);
    let data_r = body.into_inner();
    let ticket = data_r.ticket;
    req.backend
        .tickets
        .handle(&ticket, &req.instances, data_r.data);
    HttpResponse::Ok().json(true)
}

fn callback_position(
    (data, req): (web::Json<cb::TrackPositionUpdate>, web::Data<CallbackState>),
) -> HttpResponse {
    req.instances.set_pos(data.id, data.position_ms);
    spawn(
        frontend::WSServer::from_registry()
            .send(data.into_inner())
            .map_err(|e| warn!("WS-Server error: {}", e)),
    );
    HttpResponse::Ok().json(true)
}

#[derive(Clone)]
struct CallbackState {
    backend: Backend,
    instances: Instances,
}

/// Init callback server
pub fn init_callback_server(
    backend: Backend,
    instances: Instances,
    callback_server: SocketAddr,
    _tickets: super::TicketHandler,
) -> Fallible<()> {
    let state = CallbackState {
        backend: backend.clone(),
        instances,
    };
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Logger::new("manager::api::backend::callback"))
            .wrap(SecurityModule::new(backend.addr.ip()))
            .service(
                web::resource(cb::PATH_INSTANCE)
                    .data(web::JsonConfig::default().limit(4096))
                    .route(web::post().to(callback_instance)),
            )
            .service(
                web::resource(cb::PATH_VOLUME)
                    .data(web::JsonConfig::default().limit(4096))
                    .route(web::post().to(callback_volume)),
            )
            .service(
                web::resource(cb::PATH_PLAYBACK)
                    .data(web::JsonConfig::default().limit(4096))
                    .route(web::post().to(callback_playback)),
            )
            .service(
                web::resource(cb::PATH_RESOLVE)
                    .data(web::JsonConfig::default().limit(256096))
                    .route(web::post().to(callback_resolve)),
            )
            .service(
                web::resource(cb::PATH_POSITION)
                    .data(web::JsonConfig::default().limit(4096))
                    .route(web::post().to(callback_position)),
            )
    })
    .bind(callback_server)
    .map_err(|e| ServerErr::BindFailed(e))
    .unwrap()
    .shutdown_timeout(1)
    .start();

    Ok(())
}
