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

pub use server::{InstanceCreated, WSServer};

mod server;

use crate::models::UseInstance;
use actix::prelude::*;
use actix_web::{ws, Error, HttpRequest, HttpResponse};
use serde::Serialize;
use yamba_types::models::{self, ID};

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;
use server::{Connect, Disconnect, Use};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Fail, Debug)]
pub enum WsErr {
    #[fail(display = "Can't send msg to unknown instance {}", _0)]
    InvalidInstance(ID),
    #[fail(display = "Can't send msg to unknown client")]
    InvalidClient,
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
            ws::Message::Pong(_) => self.hb = Instant::now(),
            ws::Message::Text(text) => {
                if let Ok(v) = serde_json::from_str::<UseInstance>(text.as_str()) {
                    self.instance = v.id;
                    WSServer::from_registry().do_send(Use {
                        id: self.id,
                        instance: self.instance.clone(),
                    });
                }
            }
            ws::Message::Binary(_) => warn!("Unexpected binary"),
            ws::Message::Close(_) => ctx.stop(),
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
