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
    instances: Instances,
    bind_addr: SocketAddr,
) -> Fallible<ShutdownGuard> {
    let (tx, rx) = mpsc::channel(1);
    thread::spawn(move || {
        let mut sys = System::new("callback_server");
        server::new(move || {
            App::with_state(instances.clone())
                .middleware(middleware::Logger::default())
                .handler(
                    "/",
                    fs::StaticFiles::new("./templates")
                        .unwrap()
                        .index_file("index.html"),
                )
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
