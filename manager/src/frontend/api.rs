use actix::System;
use actix_web::{
    error::{InternalError, Result},
    fs, http,
    middleware::{self, Middleware, Started},
    server, App, AsyncResponder, Error, Form, HttpMessage, HttpRequest, HttpResponse, Json, State,
};
use failure::Fallible;
use futures::{
    future::{ok, result, Either},
    sync::mpsc,
    Future, Stream,
};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::thread;
use tokio::{
    executor::{DefaultExecutor, Executor, SpawnError},
    runtime::Runtime,
};
use yamba_types::models::{InstanceLoadReq, InstanceType, TSSettings};

use super::*;
use crate::backend::Backend;
use crate::instance::{Instance, Instances};

pub fn handle_create_ts(
    (state, params): (State<FrState>, Form<super::form::TSCreate>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    debug!("Form: {:?}", params);
    let mut inst_w = state.instances.write().expect("Can't lock instances!");
    let running_instance = inst_w.get(&params.id).map_or(false, |v| v.is_running());

    if running_instance {
        debug!("Instance {} already running", params.id);
        Either::B(result(Ok(HttpResponse::Conflict()
            .content_type("text/html")
            .body(format!("Instance already existing!")))))
    } else {
        // TODO: rethink verifying IP / domain
        // let ip: IpAddr = match params.ip.parse() {
        //     Err(e) => {
        //         debug!("Invalid IP {}", params.ip);
        //         return Either::B(result(Ok(HttpResponse::BadRequest()
        //             .content_type("text/html")
        //             .body(format!("Invalid IP specified!")))));
        //     }
        //     Ok(v) => v,
        // };

        let params = params.into_inner();

        let model = InstanceLoadReq {
            id: params.id,
            volume: 0.05,
            data: InstanceType::TS(TSSettings {
                host: params.host,
                port: params.port,
                identity: "".to_string(),
                cid: params.cid,
                name: params.name,
                password: None,
            }),
        };

        let instance = Instance::new(params.id, state.backend.clone(), model);

        inst_w.insert(params.id, instance);

        let inst = inst_w.get_mut(&params.id).expect("Invalid identifier ?!");

        match inst.start() {
            Ok(v) => Either::A(v.then(|res| {
                result(Ok(match res {
                    Err(e) => HttpResponse::InternalServerError()
                        .content_type("text/plain")
                        .body(format!("Error during start {:?}", e)),
                    Ok(response) => {
                        if response.success {
                            HttpResponse::Ok()
                                .content_type("text/plain")
                                .body(format!("Started instance."))
                        } else {
                            HttpResponse::InternalServerError()
                                .content_type("text/plain")
                                .body(format!("Error during start {:?}", response.msg))
                        }
                    }
                }))
            })),
            Err(e) => Either::B(result(Ok(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("Error on sending request: {}", e))))),
        }
    }
}
