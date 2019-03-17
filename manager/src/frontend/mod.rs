use crate::instance::Instance;
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

/// Guard that automatically shuts down the server on drop
pub struct ShutdownGuard {
    sender: mpsc::Sender<usize>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        let _ = self.sender.try_send(1);
    }
}

pub fn init_frontend_server(instances: &Instances) -> Fallible<ShutdownGuard> {
    let (tx, rx) = mpsc::channel(1);
    thread::spawn(move || {
        let mut sys = System::new("callback_server");
        server::new(move || {
            App::with_state(instances)
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
