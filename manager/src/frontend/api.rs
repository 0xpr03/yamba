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

use crate::models::{self, *};
use actix_web::{Error, Form, HttpResponse, Json, State};
use failure::Fallible;
use futures::{
    future::{err, result, Either},
    Future,
};
use reqwest::StatusCode;
use yamba_types::models::{InstanceLoadReq, InstanceType, Song, TSSettings};

use super::*;
use crate::instance::Instance;

/// Returns current track information
pub fn handle_instances_get(state: State<FrState>) -> Fallible<HttpResponse> {
    trace!("State..");
    Ok(HttpResponse::Ok().json(models::Instances {
        instances: state.instances.get_instances_min(),
    }))
}

/// Returns current track information
pub fn handle_track_get(
    (state, params): (State<FrState>, Json<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&params.instance) {
        let track = match i.get_current_title() {
            Some(t) => Some(TrackMin::from_song(&*t)),
            None => None,
        };
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Returns volume info
pub fn handle_volume_get(
    (state, params): (State<FrState>, Json<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&params.instance) {
        let vol = VolumeFull {
            current: i.get_volume()?,
            max: 1.0, // TODO: add support for volume limit
        };
        Ok(HttpResponse::Ok().json(vol))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Returns playback state
pub fn handle_playback_get(
    (state, params): (State<FrState>, Json<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&params.instance) {
        let playback = Playback {
            playing: i.is_playing(),
            position: i.get_pos().unwrap_or(0),
        };
        Ok(HttpResponse::Ok().json(playback))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Returns instance ID on success
pub fn handle_instances_create(
    (state, params): (State<FrState>, Json<NewInstance>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    match state
        .instances
        .create_instance(params.into_inner(), state.backend.clone())
    {
        Ok(id) => Either::A(match state.instances.read(&id).unwrap().start() {
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
                    Ok(_response) => HttpResponse::Ok().json(true),
                }))
            })),
            Err(e) => Either::B(result(Ok(HttpResponse::InternalServerError()
                .content_type("text/plain")
                .body(format!("Error on sending request: {}", e))))),
        }),
        Err(e) => {
            warn!("Error creating instance! {}", e);
            Either::B(err(e.into()))
        }
    }
}

// /// Create TS Instance
// pub fn handle_create_ts_form(
//     (state, params): (State<FrState>, Form<super::form::TSCreate>),
// ) -> impl Future<Item = HttpResponse, Error = Error> {
//     debug!("Form: {:?}", params);
//     let mut inst_w = state.instances.write().expect("Can't lock instances!");
//     let running_instance = inst_w.get(&params.id).map_or(false, |v| v.is_running());

//     if running_instance {
//         debug!("Instance {} already running", params.id);
//         Either::B(result(Ok(HttpResponse::Conflict()
//             .content_type("text/html")
//             .body(format!("Instance already existing!")))))
//     } else {
//         // TODO: rethink verifying IP / domain
//         // let ip: IpAddr = match params.ip.parse() {
//         //     Err(e) => {
//         //         debug!("Invalid IP {}", params.ip);
//         //         return Either::B(result(Ok(HttpResponse::BadRequest()
//         //             .content_type("text/html")
//         //             .body(format!("Invalid IP specified!")))));
//         //     }
//         //     Ok(v) => v,
//         // };

//         let params = params.into_inner();

//         let model = InstanceLoadReq {
//             id: params.id,
//             volume: 0.05,
//             data: InstanceType::TS(TSSettings {
//                 host: params.host,
//                 port: params.port,
//                 identity: None,
//                 cid: params.cid,
//                 name: params.name,
//                 password: None,
//             }),
//         };

//         let instance = Instance::new(params.id, state.backend.clone(), &state.instances, model);

//         inst_w.insert(params.id, instance);

//         let inst = inst_w.get_mut(&params.id).expect("Invalid identifier ?!");

//         match inst.start() {
//             Ok(v) => Either::A(v.then(|res| {
//                 result(Ok(match res {
//                     Err(e) => match e.status() {
//                         Some(StatusCode::CONFLICT) => HttpResponse::Conflict()
//                             .content_type("text/plain")
//                             .body(format!("Instance already started! {:?}", e)),
//                         _ => HttpResponse::InternalServerError()
//                             .content_type("text/plain")
//                             .body(format!("Error during start {:?}", e)),
//                     },
//                     Ok(_response) => HttpResponse::Ok()
//                         .content_type("text/plain")
//                         .body(format!("Started instance.")),
//                 }))
//             })),
//             Err(e) => Either::B(result(Ok(HttpResponse::InternalServerError()
//                 .content_type("text/plain")
//                 .body(format!("Error on sending request: {}", e))))),
//         }
//     }
// }
