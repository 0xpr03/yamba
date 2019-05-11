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

use failure::Fallible;
use hashbrown::HashMap;
use yamba_types::models::{callback::*, Ticket as TicketID, ID};

use std::sync::{Arc, RwLock};

use crate::db::Database;
use crate::instance::Instances;
use crate::models::{NewPlaylistData, PlaylistData};

/// Ticket Handler that stores callback tickets.  
/// Knows action to perform for specific IDs.
#[derive(Clone)]
pub struct TicketHandler {
    data: Arc<RwLock<HashMap<TicketID, Box<Ticket + Send + Sync>>>>,
}

impl TicketHandler {
    pub fn new() -> TicketHandler {
        TicketHandler {
            data: Arc::new(RwLock::new(
                HashMap::<TicketID, Box<Ticket + Send + Sync>>::new(),
            )),
        }
    }

    /// Add queue ticket
    pub fn add_queue(&self, instance: ID, ticket: TicketID) {
        let mut data_w = self.data.write().expect("Can't lock tickets!");
        let handler = QueueTicket::new(instance);
        data_w.insert(ticket, Box::new(handler));
    }

    /// Handle ticket
    pub fn handle(&self, ticket: &TicketID, instances: &Instances, data: ResolveResponseData) {
        let mut data_w = self.data.write().expect("Can't lock tickets!");
        debug!("Handling {}", ticket);
        match data_w.get_mut(ticket) {
            Some(v) => match v.handle(instances, data) {
                Err(e) => warn!("Error on handling ticket: {}", e),
                Ok(true) => {
                    data_w.remove(ticket);
                }
                _ => (),
            },
            None => error!("Ticket unknown: {} {:?}!", ticket, data),
        }
    }
}

/// Ticket with action desciption
pub trait Ticket {
    /// Return: Whether ticket is finished or not
    fn handle(&mut self, instances: &Instances, data: ResolveResponseData) -> Fallible<bool>;
}

/// Queue ticket type, inserts into queue
pub struct QueueTicket {
    instance: ID,
    playlist: Option<PlaylistData>,
}

impl QueueTicket {
    pub fn new(instance: ID) -> QueueTicket {
        QueueTicket {
            instance,
            playlist: None,
        }
    }
}

impl QueueTicket {
    fn part(&mut self, instances: &Instances, data: ResolvePartResponse) -> Fallible<bool> {
        let p = data.data;
        for song in &p.songs {
            instances
                .get_db()
                .upsert_song(song, &Some(p.source.as_str()))?;
        }
        if let Some(ref mut playlist) = self.playlist {
            playlist.data.append(&mut p.songs.clone());
            if data.state == ResolveState::Finished {
                instances
                    .get_db()
                    .upsert_playlist(&NewPlaylistData::from_playlist(playlist))?;
            }
        } else {
            //TODO: rethink adding support for out-of-order callbacks
            panic!("Expected playlist on part-resolve, no playlist found. Part response before initial!?")
        }

        instances.read(&self.instance).map(|inst| {
            inst.add_to_queue(p.songs);
            inst.check_playback();
        });
        Ok(data.state == ResolveState::Finished)
    }
    fn initial(&mut self, instances: &Instances, data: ResolveInitialResponse) -> Fallible<bool> {
        let songs = match data.data {
            ResolveType::Song(s) => {
                instances
                    .get_db()
                    .upsert_song(&s, &Some(s.source.as_str()))?;
                let mut ret = Vec::with_capacity(1);
                ret.push(s);
                ret
            }
            ResolveType::Playlist(p) => {
                for song in &p.songs {
                    instances
                        .get_db()
                        .upsert_song(song, &Some(p.source.as_str()))?;
                }
                let pl_data =
                    PlaylistData::new(p.name, Some(p.source), p.songs.clone(), instances.get_db())?;
                instances
                    .get_db()
                    .upsert_playlist(&NewPlaylistData::from_playlist(&pl_data))?;
                self.playlist = Some(pl_data);
                p.songs
            }
        };
        instances.read(&self.instance).map(|inst| {
            inst.add_to_queue(songs);
            inst.check_playback();
        });

        Ok(data.state == ResolveState::Finished)
    }
    fn error(&mut self, err: ResolveErrorResponse) -> Fallible<bool> {
        warn!(
            "Error on resolving song: {}, {:#?}",
            err.msg.unwrap_or(String::from("No Message")),
            err.details
        );
        Ok(true)
    }
}

impl Ticket for QueueTicket {
    fn handle(&mut self, instances: &Instances, data: ResolveResponseData) -> Fallible<bool> {
        match data {
            ResolveResponseData::Part(part) => self.part(instances, part),
            ResolveResponseData::Error(err) => self.error(err),
            ResolveResponseData::Initial(inital) => self.initial(instances, inital),
        }
    }
}
