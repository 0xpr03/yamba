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

use actix_web::{Error, Form, HttpResponse, State};
use futures::{
    future::{result, Either},
    Future,
};
use reqwest::StatusCode;
use yamba_types::models::{InstanceLoadReq, InstanceType, TSSettings};

use super::*;
use crate::instance::Instance;

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
                    Err(e) => match e.status() {
                        Some(StatusCode::CONFLICT) => HttpResponse::Conflict()
                            .content_type("text/plain")
                            .body(format!("Instance already started! {:?}", e)),
                        _ => HttpResponse::InternalServerError()
                            .content_type("text/plain")
                            .body(format!("Error during start {:?}", e)),
                    },
                    Ok(_response) => HttpResponse::Ok()
                        .content_type("text/plain")
                        .body(format!("Started instance.")),
                }))
            })),
            Err(e) => Either::B(result(Ok(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("Error on sending request: {}", e))))),
        }
    }
}
