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

use jsonrpc_lite::{Error, Id, JsonRpc, Params};

use failure::Fallible;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use serde_json::{self, to_value, Value};
use tokio::runtime;

use daemon::{self, BoxFut, InstanceType, Instances};
use SETTINGS;

/// RPC server for client callbacks

#[derive(Fail, Debug)]
pub enum RPCErr {
    #[fail(display = "RPC bind error {}", _0)]
    BindError(#[cause] hyper::error::Error),
}

fn rpc(req: Request<Body>, instances: Instances) -> BoxFut {
    Box::new(req.into_body().concat2().map(move |b| {
        let response_rpc = if let Ok(rpc) = serde_json::from_slice::<JsonRpc>(&b) {
            trace!("rpc request: {:?}", rpc);
            let mut req_id = rpc.get_id().unwrap_or(Id::None(()));
            match parse_rpc_call(req_id.clone(), &rpc) {
                Ok((instance_id, method, params)) => match method {
                    "heartbeat" => JsonRpc::success(req_id, &json!(true)),
                    "connected" => {
                        trace!("ts connected");
                        let instance_r = instances.read().expect("Can't read instance!");
                        if let Some(instance) = instance_r.get(&instance_id) {
                            // if let, but irrefutable pattern as of now..
                            let InstanceType::Teamspeak(ref ts) = instance.voip;
                            ts.on_connected();

                            JsonRpc::success(req_id, &json!(true))
                        } else {
                            error!(
                                "Received connected event for invalid instance ID {:?}",
                                req_id
                            );
                            JsonRpc::error(req_id, Error::invalid_params())
                        }
                    }
                    _ => JsonRpc::error(req_id, Error::method_not_found()),
                },
                Err(e) => {
                    warn!("Can't parse rpc: {:?}", e);
                    e
                }
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

/// Parse input and retrieve relevant data
fn parse_rpc_call(req_id: Id, rpc: &JsonRpc) -> Result<(i32, &str, Vec<Value>), JsonRpc> {
    let params = match rpc.get_params() {
        Some(Params::Array(params)) => params,
        v => {
            warn!("Invalid rpc request params: {:?}", v);
            return Err(JsonRpc::error(Id::None(()), Error::parse_error()));
        }
    };
    let instance_id = match params.get(0) {
        Some(Value::Number(id)) => {
            if let Some(id) = id.as_i64() {
                id as i32
            } else {
                return Err(JsonRpc::error(req_id, Error::invalid_params()));
            }
        }
        v => {
            warn!("Missing instance ID for rpc call! {:?}", v);
            return Err(JsonRpc::error(req_id, Error::invalid_request()));
        }
    };
    let method = match rpc.get_method() {
        Some(method) => method,
        _ => return Err(JsonRpc::error(req_id, Error::invalid_request())),
    };
    Ok((instance_id as i32, method, params))
}

/// Check config for RPC, throws an error if not ok for starting
pub fn check_config() -> Fallible<()> {
    let _ = daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;
    Ok(())
}

/// Create rpc server, bind it & attach to runtime
pub fn create_rpc_server(runtime: &mut runtime::Runtime, instances: Instances) -> Fallible<()> {
    let addr =
        daemon::parse_socket_address(&SETTINGS.main.rpc_bind_ip, SETTINGS.main.rpc_bind_port)?;
    let server = Server::try_bind(&addr)
        .map_err(|e| RPCErr::BindError(e))?
        .serve(move || {
            let instances = instances.clone();
            service_fn(move |req: Request<Body>| rpc(req, instances.clone()))
        })
        .map_err(|e| error!("server error: {}", e));

    info!("RPC Listening on http://{}", addr);
    runtime.spawn(server);
    Ok(())
}
