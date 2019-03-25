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

use crate::instance::Instances;

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
    pub fn handle(&self, ticket: &TicketID, instances: &Instances, songs: Vec<Song>) {
        let mut data_w = self.data.write().expect("Can't lock tickets!");
        debug!("Handling {}", ticket);
        match data_w.remove(ticket) {
            Some(v) => {
                v.handle(instances, songs);
            }
            None => warn!("Ticket unknown: {} {:?}!", ticket, songs),
        }
    }
}

/// Ticket with action desciption
pub trait Ticket {
    fn handle(&self, instances: &Instances, songs: Vec<Song>) -> Fallible<()>;
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
    fn handle(&self, instances: &Instances, songs: Vec<Song>) -> Fallible<()> {
        let inst_r = instances.read().expect("Can't lock instances!");
        inst_r.get(&self.instance).map(|inst| {
            inst.add_to_queue(songs);
            inst.check_playback();
        });

        Ok(())
    }
}

/*
pub struct ResolveTicket {
    id: TicketID,
    playlist: Option<i32>,
}
*/
