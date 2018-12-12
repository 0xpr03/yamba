/*
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */

use jsonrpc_lite::{Error, Id, JsonRpc};

use daemon::{self, BoxFut};
use failure::Fallible;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use serde_json::{self, to_value};
use tokio::runtime;
use SETTINGS;

/// RPC server for client callbacks

#[derive(Fail, Debug)]
pub enum RPCErr {
    #[fail(display = "RPC bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

fn rpc(req: Request<Body>) -> BoxFut {
    Box::new(req.into_body().concat2().map(|b| {
        let response_rpc = if let Ok(rpc) = serde_json::from_slice::<JsonRpc>(&b) {
            trace!("rpc request: {:?}", rpc);
            let id = rpc.get_id().unwrap_or(Id::None(()));
            if let Some(method) = rpc.get_method() {
                match method {
                    "heartbeat" => JsonRpc::success(id, &json!(true)),
                    _ => JsonRpc::error(id, Error::method_not_found()),
                }
            } else {
                JsonRpc::error(id, Error::invalid_request())
            }
        } else {
            warn!("Invalid rpc request");
            JsonRpc::error(Id::None(()), Error::parse_error())
        };
        // https://github.com/hyperium/hyper/blob/master/examples/params.rs
        let body = to_value(response_rpc).unwrap().to_string();
        Response::new(body.into())
    }))
}

/// Check config for RPC, throws an error if not ok for starting
pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;
    Ok(())
}

/// Create rpc server, bind it & attach to runtime
pub fn create_rpc_server(runtime: &mut runtime::Runtime) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;

    let server = Server::try_bind(&addr)
        .map_err(|e| RPCErr::BindError(e))?
        .serve(|| service_fn(rpc))
        .map_err(|e| eprintln!("server error: {}", e));

    info!("RPC Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}
