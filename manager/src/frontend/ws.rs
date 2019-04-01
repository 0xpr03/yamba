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
use actix_web::{ws, Error, HttpRequest, HttpResponse};
use rand::{self, rngs::ThreadRng, Rng};
use serde::Serialize;
use yamba_types::models::{self, ID};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Fail, Debug)]
pub enum WsErr {
    #[fail(display = "Can't send msg to unknown instance {}", _0)]
    InvalidInstance(ID),
}

#[derive(Message, Serialize)]
pub enum Message {
    VolumeChange(models::VolumeSetReq),
    InstancePlayback(models::callback::PlaystateResponse),
    InstanceCreated(ID),
    PositionUpdate(models::callback::TrackPositionUpdate),
}

#[derive(Serialize)]
pub enum ClientMessage {
    VolumeCHange(models::VolumeSetReq),
}

struct WsSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    instance: ID,
}

/// Raw Message of a String which allows seriializing once and sending to multiple recipients
#[derive(Message, Clone)]
pub struct RawMessage(pub Arc<String>);

impl RawMessage {
    pub fn new(msg: &Message) -> RawMessage {
        RawMessage(Arc::new(serde_json::to_string(msg).unwrap()))
    }
}

impl Handler<RawMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: RawMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self, FrState>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        WSServer::from_registry()
            .send(Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // TODO: remove client from internal state ?
        WSServer::from_registry().do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/join" => {
                            if v.len() == 2 {
                                self.instance = v[1].parse::<ID>().unwrap();
                                WSServer::from_registry().do_send(Use {
                                    id: self.id,
                                    instance: self.instance.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    // let msg = if let Some(ref name) = self.name {
                    //     format!("{}: {}", name, m)
                    // } else {
                    //     m.to_owned()
                    // };
                    // // send message to chat server
                    // ctx.state().ws.do_send(ClientMessage {
                    //     id: self.id,
                    //     msg: msg,
                    //     instance: self.instance.clone(),
                    // })
                }
            }
            ws::Message::Binary(_) => warn!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self, FrState>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                WSServer::from_registry().do_send(Disconnect { id: act.id });
                ctx.stop();
                return;
            }

            ctx.ping("");
        });
    }
}

pub fn ws_route(req: &HttpRequest<FrState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsSession {
            id: 0,
            hb: Instant::now(),
            instance: 1,
        },
    )
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
                debug!("Evaluating to {}", id);
                if *id != skip_id {
                    debug!("Sending to {}", id);
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
        debug!("Volume changed {}", msg.id);

        self.send_message(&msg.id.clone(), &Message::VolumeChange(msg), 0);
    }
}

/// Internal, Send instance track position update
impl Handler<models::callback::TrackPositionUpdate> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: models::callback::TrackPositionUpdate, _: &mut Context<Self>) {
        debug!("Volume changed {}", msg.id);

        self.send_message(&msg.id.clone(), &Message::PositionUpdate(msg), 0);
    }
}

/// Internal, Send instance playback state
impl Handler<models::callback::PlaystateResponse> for WSServer {
    type Result = ();

    fn handle(&mut self, msg: models::callback::PlaystateResponse, _: &mut Context<Self>) {
        debug!("Volume changed {}", msg.id);

        self.send_message(&msg.id.clone(), &Message::InstancePlayback(msg), 0);
    }
}

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

/// Select instance to use
#[derive(Message)]
pub struct Use {
    /// Client id
    pub id: usize,
    /// Room name
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

        if self.instances.get_mut(&instance).is_none() {
            //TODO: handle invalid instance
            // self.instances.insert(name.clone(), HashSet::new());
        }
        self.instances.get_mut(&instance).unwrap().insert(id);
    }
}
