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
use yamba_types::models::{Song, Ticket as TicketID, ID};

use std::sync::{Arc, RwLock};

use crate::db::Database;
use crate::instance::Instances;
use crate::models::NewPlaylistData;

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
    pub fn handle(
        &self,
        ticket: &TicketID,
        instances: &Instances,
        songs: Vec<Song>,
        source: String,
    ) {
        let mut data_w = self.data.write().expect("Can't lock tickets!");
        debug!("Handling {}", ticket);
        match data_w.remove(ticket) {
            Some(v) => {
                if let Err(e) = v.handle(instances, songs, source) {
                    warn!("Error on handling ticket: {}", e);
                }
            }
            None => warn!("Ticket unknown: {} {:?}!", ticket, songs),
        }
    }
}

/// Ticket with action desciption
pub trait Ticket {
    fn handle(&self, instances: &Instances, songs: Vec<Song>, souce: String) -> Fallible<()>;
}

/// Queue ticket type, inserts into queue
pub struct QueueTicket {
    instance: ID,
}

impl QueueTicket {
    pub fn new(instance: ID) -> QueueTicket {
        QueueTicket { instance }
    }
}

impl Ticket for QueueTicket {
    fn handle(&self, instances: &Instances, songs: Vec<Song>, source: String) -> Fallible<()> {
        let song_url = match songs.len() == 1 {
            true => Some(source.as_str()),
            false => None,
        };
        for song in &songs {
            instances.get_db().upsert_song(song, &song_url)?;
        }
        if songs.len() > 1 {
            let pl_data = NewPlaylistData::new(String::new(), &songs, instances.get_db())?;
            instances
                .get_db()
                .upsert_playlist(&pl_data, Some(source.as_str()))?;
        }
        instances.read(&self.instance).map(|inst| {
            inst.add_to_queue(songs);
            inst.check_playback();
        });

        Ok(())
    }
}
