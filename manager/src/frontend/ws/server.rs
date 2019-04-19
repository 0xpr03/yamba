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

use actix::prelude::*;
use actix::registry::SystemService;
use failure::Fallible;
use rand::{self, rngs::ThreadRng, Rng};
use yamba_types::models::{self, ID};

use std::collections::{HashMap, HashSet};

use super::{Message, RawMessage, WsErr};
use crate::instance::Instances;

/// Print warning on error in send_message
macro_rules! warn_log {
    ($x:expr) => {
        if let Err(e) = $x {
            warn!("Can't push to WS: {}", e);
        }
    };
}

/// WebsocketServer - holds all connected clients
pub struct WSServer {
    sessions: HashMap<usize, Recipient<RawMessage>>,
    instances: HashMap<ID, HashSet<usize>>,
    rng: ThreadRng,
}

impl SystemService for WSServer {}
impl Supervised for WSServer {}

impl WSServer {
    /// Send message to all clients for an instance
    fn send_message(&self, instance: &ID, message: &Message, skip_id: usize) -> Fallible<()> {
        if let Some(sessions) = self.instances.get(instance) {
            let msg = RawMessage::new(message);
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(msg.clone());
                    }
                }
            }

            Ok(())
        } else {
            Err(WsErr::InvalidInstance(instance.clone()).into())
        }
    }

    /// Send message to specific client
    fn send_direct_message(&self, client_id: &usize, message: RawMessage) -> Fallible<()> {
        if let Some(client) = self.sessions.get(client_id) {
            client.do_send(message)?;
            Ok(())
        } else {
            Err(WsErr::InvalidClient.into())
        }
    }

    /// Send global message to all clients
    fn send_global_message(&self, message: &Message) {
        for (id, client) in self.sessions.iter() {
            debug!("Sending to {:?}", id);
            let _ = client.do_send(RawMessage::new(message));
        }
    }
}

impl Default for WSServer {
    fn default() -> WSServer {
        // default room

        let mut e = HashMap::new();
        e.insert(1, HashSet::new());

        WSServer {
            sessions: HashMap::new(),
            instances: e,
            rng: rand::thread_rng(),
        }
    }
}

impl Actor for WSServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<RawMessage>,
}

impl Handler<Connect> for WSServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to Main room
        self.instances.get_mut(&1).unwrap().insert(id);

        // send id back
        id
    }
}

/// Internal: Send on instance creation
#[derive(Message)]
pub struct InstanceCreated {
    pub id: ID,
}

impl Handler<InstanceCreated> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: InstanceCreated, _: &mut Context<Self>) {
        debug!("Instance created {}", msg.id);

        self.instances.insert(msg.id, HashSet::new());

        // send message to other users
        self.send_global_message(&Message::InstanceCreated(msg.id));
    }
}

/// Internal: Send instance volume change
impl Handler<models::VolumeSetReq> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: models::VolumeSetReq, _: &mut Context<Self>) {
        debug!("Volume changed on {}", msg.id);

        warn_log!(self.send_message(&msg.id.clone(), &Message::VolumeChange(msg), 0));
    }
}

/// Internal, Send instance track position update
impl Handler<models::callback::TrackPositionUpdate> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: models::callback::TrackPositionUpdate, _: &mut Context<Self>) {
        trace!("Position changed on {}", msg.id);

        warn_log!(self.send_message(&msg.id.clone(), &Message::PositionUpdate(msg), 0));
    }
}

/// Internal, Send instance playback state
impl Handler<models::callback::PlaystateResponse> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: models::callback::PlaystateResponse, _: &mut Context<Self>) {
        debug!("Volume changed {}", msg.id);

        warn_log!(self.send_message(&msg.id.clone(), &Message::InstancePlayback(msg), 0));
    }
}

/// Client Disconnect
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

impl Handler<Disconnect> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Someone disconnected");

        let mut instances: Vec<ID> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (id, sessions) in &mut self.instances {
                if sessions.remove(&msg.id) {
                    instances.push(id.to_owned());
                }
            }
        }
    }
}

/// Client select instance to use
#[derive(Message)]
pub struct Use {
    /// Client id
    pub id: usize,
    /// Instance id
    pub instance: ID,
}

impl Handler<Use> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: Use, _: &mut Context<Self>) {
        let Use { id, instance } = msg;
        let mut instances = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.instances {
            if sessions.remove(&id) {
                instances.push(n.to_owned());
            }
        }

        if let Some(i) = self.instances.get_mut(&instance) {
            i.insert(id);

        // let state = InitialState {
        //     track: self.
        // };

        // warn_log!(self.send_direct_message(&id, RawMessage::from_serializable(&state)));
        } else {
            //TODO: handle invalid instance
            // self.instances.insert(name.clone(), HashSet::new());
        }
    }
}
