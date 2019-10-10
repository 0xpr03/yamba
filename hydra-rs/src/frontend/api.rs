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

use super::*;
use crate::instance::InstanceErr;
use crate::models::{self, GenericRequest};
use actix_web::{web, Error, HttpResponse};
use failure::Fallible;
use futures::{
    future::{err, result, Either},
    Future,
};

use reqwest::StatusCode;

/// Stop running instance
pub fn handle_instance_stop(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    if let Some(i) = state.instances.read(&path.instance) {
        match i.stop() {
            Ok(res) => Either::A(res.then(|_| result(Ok(HttpResponse::Ok().json(true))))),
            Err(e) => Either::B(Either::A(err(e.into()))),
        }
    } else {
        Either::B(Either::B(result(Ok(
            HttpResponse::BadRequest().json("Invalid instance!")
        ))))
    }
}

/// Start existing instance
pub fn handle_instance_start(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> impl Future<Item = HttpResponse, Error = Error> {
    match state.instances.start_instance(path.instance) {
        Ok(v) => Either::A(v.then(|res| {
            result(Ok(match res {
                Err(e) => match e.status() {
                    Some(StatusCode::CONFLICT) => HttpResponse::Conflict()
                        .content_type("text/plain")
                        .body("Instance already started!"),
                    _ => {
                        error!("Error starting instance: {}", e);
                        HttpResponse::InternalServerError().finish()
                    }
                },
                Ok(_response) => HttpResponse::Ok().json(true),
            }))
        })),
        Err(e) => {
            error!("Error starting instance: {}", e);
            Either::B(result(Ok(HttpResponse::InternalServerError().finish())))
        }
    }
}

/// Delete instance
pub fn handle_instance_delete(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    match state.instances.delete_instance(&path.instance) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => match e.downcast_ref::<InstanceErr>() {
            Some(InstanceErr::InstanceRunning) => {
                Ok(HttpResponse::BadRequest().json("Instance still running!"))
            }
            Some(InstanceErr::NoInstanceFound(_)) => {
                Ok(HttpResponse::BadRequest().json("Invalid instance!"))
            }
            _ => {
                error!("Unable to delete instance: {}", e);
                Err(e)
            }
        },
    }
}

/// Returns instance core configuration
pub fn handle_instance_config_core_get(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
        let details: models::InstanceCoreRef = i.get_core_config();
        Ok(HttpResponse::Ok().json(details))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Update instance core configuration
pub fn handle_instance_config_core_update(
    (state, path, data): (
        web::Data<FrState>,
        web::Path<GenericRequest>,
        web::Json<models::InstanceCore>,
    ),
) -> Fallible<HttpResponse> {
    trace!("Update instance core {}", path.instance);
    if let Some(mut i) = state.instances.read_mut(&path.instance) {
        i.update_core_config(data.into_inner())?;
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Returns current track information
pub fn handle_instances_get(state: web::Data<FrState>) -> Fallible<HttpResponse> {
    trace!("Instances get..");
    let instances: models::Instances = state.instances.get_instances_min();
    Ok(HttpResponse::Ok().json(instances))
}

/// Returns current track information
pub fn handle_track_get(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
        let track = match i.get_current_title() {
            Some(t) => Some(models::TrackMin::from_song(&*t)),
            None => None,
        };
        Ok(HttpResponse::Ok().json(track))
    } else {
        Ok(HttpResponse::BadRequest().json("Invalid instance!"))
    }
}

/// Returns volume info
pub fn handle_volume_get(
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
        let vol = models::VolumeFull {
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
    (state, path): (web::Data<FrState>, web::Path<GenericRequest>),
) -> Fallible<HttpResponse> {
    if let Some(i) = state.instances.read(&path.instance) {
        let playback = models::Playback {
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
    (state, params): (web::Data<FrState>, web::Json<models::InstanceCore>),
) -> Fallible<HttpResponse> {
    match state
        .instances
        .create_instance(params.into_inner(), state.backend.clone())
    {
        Ok(id) => Ok(HttpResponse::Created().json(id)),
        Err(e) => {
            warn!("Error creating instance! {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
