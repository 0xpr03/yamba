use super::*;
use actix::System;
use actix_web::{
    error::Result,
    http,
    middleware::{self, Middleware, Started},
    server, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse, Json,
};
use failure::Fallible;
use futures::{sync::mpsc, Future, Stream};
use std::net::IpAddr;
use std::thread;
use tokio::{
    executor::{DefaultExecutor, Executor, SpawnError},
    runtime::Runtime,
};
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
            if remote == self.ip {
                return Ok(Started::Done);
            }
        }
        Ok(Started::Response(HttpResponse::Unauthorized().finish()))
    }
}

/// Handle instance callback
fn callback_instance(
    (data, req): (Json<cb::InstanceStateResponse>, HttpRequest<Backend>),
) -> HttpResponse {
    debug!("Instance state change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        i.set_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_volume(
    (data, req): (Json<cb::InstanceStateResponse>, HttpRequest<Backend>),
) -> HttpResponse {
    debug!("Volume change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        i.set_state(data.into_inner().state);
    }
    HttpResponse::Ok().json(true)
}

fn callback_playback(
    (data, req): (Json<cb::PlaystateResponse>, HttpRequest<Backend>),
) -> HttpResponse {
    debug!("Volume change: {:?}", data);
    let inst = req.state().instances.read().expect("Can't lock instances!");
    if let Some(i) = inst.get(&data.id) {
        match &data.state {
            cb::Playstate::EndOfMedia => i.song_end(),
            v => debug!("Playback change: {:?}", v),
        }
    }
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

/// Init callback server
pub fn init_callback_server(backend: Backend) -> Fallible<ShutdownGuard> {
    let (tx, rx) = mpsc::channel(1);
    thread::spawn(move || {
        let mut sys = System::new("callback_server");
        server::new(move || {
            App::with_state(backend.clone())
                .middleware(middleware::Logger::default())
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
        })
        .bind("127.0.0.1:8080")
        .map_err(|e| ServerErr::BindFailed(e))
        .unwrap()
        .shutdown_timeout(1)
        .run();

        sys.block_on(rx.into_future().map(|_| println!("received shutdown")))
            .unwrap();
    });

    Ok(ShutdownGuard { sender: tx })
}
