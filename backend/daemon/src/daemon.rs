use failure::{self, Fallible};
use futures::{Future, Stream};
use hyper::{self, Body, Response};
use tokio::{self, runtime::Runtime};
use tokio_signal;

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
    ShutdownError,
}

/// Start runtime
pub fn start_runtime() -> Fallible<()> {
    let mut rt = Runtime::new().map_err(|e| DaemonErr::RuntimeCreationError(e))?;
    rpc::create_rpc_server(&mut rt).map_err(|e| DaemonErr::RPCCreationError(e))?;
    api::create_api_server(&mut rt).map_err(|e| DaemonErr::APICreationError(e))?;
    register_sighub(&mut rt);
    /*let ctrl_c = tokio_signal::ctrl_c().flatten_stream();
    let runtime = Arc::new(Mutex::new(Some(rt)));
    // Process each ctrl-c as it comes in
    let prog = ctrl_c.for_each(move |()| {
        let mut data = runtime.lock().unwrap();
        if let Some(rt) = data.take() {
            rt.shutdown_now();
            info!("Server stopped");
        } else {
            warn!("Can't shutdown server (already stopped?)");
        }
        Ok(())
    });
    let mut rt = runtime.lock().unwrap();
    rt.as_mut().unwrap().block_on(prog);*/
    rt.shutdown_on_idle()
        .wait()
        .map_err(|_| DaemonErr::ShutdownError)?;
    Ok(())
}

fn register_sighub(rt: &mut Runtime) {}

/// Parse socket address
pub fn parse_socket_address(ip: &str, port: u16) -> Fallible<SocketAddr> {
    Ok(SocketAddr::new(ip.parse()?, port))
}
