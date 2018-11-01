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

use futures::future;
use hyper;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use serde_json::{self, to_value};

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn echo(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

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

/// Starts rpc daemon
pub fn run_rpc_daemon() {
    let addr = ([127, 0, 0, 1], 1337).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(echo))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
