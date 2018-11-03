use failure::{self, Fallible};
use futures::{Future, Stream};
use hyper::{self, Body, Response};
use tokio::{self, runtime::Runtime};
use tokio_signal::unix::{self, Signal};

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use api;
use rpc;

/// Daemon init & handling

// type used by rpc & api
pub type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

#[derive(Fail, Debug)]
pub enum DaemonErr {
    #[fail(display = "Unable to open default config {}", _0)]
    RuntimeCreationError(#[cause] tokio::io::Error),
    #[fail(display = "Unable to create rpc server {}", _0)]
    RPCCreationError(#[cause] failure::Error),
    #[fail(display = "Unable to create api server {}", _0)]
    APICreationError(#[cause] failure::Error),
    #[fail(display = "Error on shutdown of runtime")]
    ShutdownError(#[cause] io::Error),
}

/// Start runtime
pub fn start_runtime() -> Fallible<()> {
    let mut rt = Runtime::new().map_err(|e| DaemonErr::RuntimeCreationError(e))?;
    rpc::create_rpc_server(&mut rt).map_err(|e| DaemonErr::RPCCreationError(e))?;
    api::create_api_server(&mut rt).map_err(|e| DaemonErr::APICreationError(e))?;
    info!("Daemon initialized");
    //let stream = ctrl_c.wait()?;
    match rt.block_on(
        Signal::new(unix::libc::SIGINT)
            .flatten_stream()
            .into_future()
            .select(
                Signal::new(unix::libc::SIGTERM)
                    .flatten_stream()
                    .into_future(),
            ),
    ) {
        Err(e) => {
            let ((err, _), _) = e;
            return Err(DaemonErr::ShutdownError(err).into());
        }
        Ok(_) => (),
    };
    Ok(())
}

/// Parse socket address
pub fn parse_socket_address(ip: &str, port: u16) -> Fallible<SocketAddr> {
    Ok(SocketAddr::new(ip.parse()?, port))
}
