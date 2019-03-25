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
use actix::System;
use actix_web::{
    error::Result,
    http,
    middleware::{self, Middleware, Started},
    server, App, HttpRequest, HttpResponse, Json,
};
use failure::Fallible;
use futures::{sync::mpsc, Future, Stream};
use std::net::IpAddr;
use std::thread;
use yamba_types::models::callback as cb;

#[derive(Fail, Debug)]
pub enum ServerErr {
    #[fail(display = "Failed to bind callback server {}", _0)]
    BindFailed(#[cause] std::io::Error),
}

struct SecurityModule {
    ip: String,
}

impl SecurityModule {
    pub fn new(addr: IpAddr) -> SecurityModule {
        SecurityModule {
            ip: addr.to_string(),
        }
    }
}

impl<S> Middleware<S> for SecurityModule {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started> {
        if let Some(remote) = req.connection_info().remote() {
            if remote
                .parse::<SocketAddr>()
                .map(|v| v.ip().to_string() == self.ip)
                .unwrap_or_else(|e| {
                    warn!("Can't parse remote IP! {}", e);
                    false
                })
            {
                return Ok(Started::Done);
            } else {
                debug!("Remote: {} Own: {}", remote, self.ip);
            }
        }
        Ok(Started::Response(HttpResponse::Unauthorized().finish()))
    }
}

/// Handle instance callback
fn callback_instance(
    (data, req): (Json<cb::InstanceStateResponse>, HttpRequest<CallbackState>),
) -> HttpResponse {
    debug!("Instance state change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        i.cb_set_instance_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_volume(
    (data, req): (Json<cb::VolumeChange>, HttpRequest<CallbackState>),
) -> HttpResponse {
    debug!("Volume change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        i.cb_update_volume(data.into_inner().volume);
    }
    HttpResponse::Ok().json(true)
}

fn callback_playback(
    (data, req): (Json<cb::PlaystateResponse>, HttpRequest<CallbackState>),
) -> HttpResponse {
    debug!("Playback change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        i.cb_set_playback_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_resolve(
    (data, req): (Json<cb::ResolveResponse>, HttpRequest<CallbackState>),
) -> HttpResponse {
    debug!("Resolve callback: {:?}", data);
    let data_r = data.into_inner();
    let ticket = data_r.ticket;
    let songs = data_r.songs;
    req.state()
        .backend
        .tickets
        .handle(&ticket, &req.state().instances, songs);
    HttpResponse::Ok().json(true)
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
) -> Fallible<ShutdownGuard> {
    let state = CallbackState {
        backend: backend.clone(),
        instances,
    };

    let (tx, rx) = mpsc::channel(1);
    thread::spawn(move || {
        let mut sys = System::new("callback_server");
        server::new(move || {
            App::with_state(state.clone())
                .middleware(middleware::Logger::new("manager::api::backend::callback"))
                .middleware(SecurityModule::new(backend.addr.ip()))
                .resource(cb::PATH_INSTANCE, |r| {
                    r.method(http::Method::POST)
                        .with_config(callback_instance, |((cfg, _),)| {
                            cfg.limit(4096);
                        })
                })
                .resource(cb::PATH_VOLUME, |r| {
                    r.method(http::Method::POST)
                        .with_config(callback_volume, |((cfg, _),)| {
                            cfg.limit(4096);
                        })
                })
                .resource(cb::PATH_PLAYBACK, |r| {
                    r.method(http::Method::POST)
                        .with_config(callback_playback, |((cfg, _),)| {
                            cfg.limit(4096);
                        })
                })
                .resource(cb::PATH_RESOLVE, |r| {
                    r.method(http::Method::POST)
                        .with_config(callback_resolve, |((cfg, _),)| {
                            cfg.limit(256096);
                        })
                })
        })
        .bind(callback_server)
        .map_err(|e| ServerErr::BindFailed(e))
        .unwrap()
        .shutdown_timeout(1)
        .run();

        sys.block_on(rx.into_future().map(|_| println!("received shutdown")))
            .unwrap();
    });

    Ok(ShutdownGuard { sender: tx })
}
