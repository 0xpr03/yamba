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
use actix_web::{Error, HttpResponse, Json, Path, State};
use failure::Fallible;
use futures::{
    future::{err, result, Either},
    Future,
};
use reqwest::StatusCode;

use super::*;

/// Stop running instance
pub fn handle_instance_stop(
    (state, path): (State<FrState>, Path<GenericRequest>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    if let Some(i) = state.instances.read(&path.instance) {
        match i.stop() {
            Ok(res) => Either::A(res.then(|res| result(Ok(HttpResponse::Ok().json(true))))),
            Err(e) => Either::B(Either::A(err(e.into()))),
        }
    } else {
        Either::B(Either::B(result(Ok(
            HttpResponse::BadRequest().json("Invalid instance!")
        ))))
    }
}

/// Returns current track information
pub fn handle_instances_get(state: State<FrState>) -> Fallible<HttpResponse> {
    trace!("Instances get..");
    let instances: models::Instances = state.instances.get_instances_min();
    Ok(HttpResponse::Ok().json(instances))
}

/// Returns current track information
pub fn handle_track_get(
    (state, path): (State<FrState>, Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
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
    (state, path): (State<FrState>, Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
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
    (state, path): (State<FrState>, Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
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
        Ok(id) => Either::A(match state.instances.start_instance(id) {
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
