use actix::prelude::*;
use actix::{registry::SystemService, *};
use actix_web::{ws, Error, HttpRequest, HttpResponse};
use rand::{self, rngs::ThreadRng, Rng};
use yamba_types::models::ID;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use super::*;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub type WSState = Addr<WSServer>;

struct WsSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT),
    /// otherwise we drop connection.
    hb: Instant,
    /// joined room
    instance: ID,
}

impl Handler<Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
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

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
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
            ws::Message::Binary(bin) => println!("Unexpected binary"),
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

#[derive(Message)]
pub struct Message(pub String);

/// WebsocketServer - holds all connected clients
pub struct WSServer {
    sessions: HashMap<usize, Recipient<Message>>,
    instances: HashMap<ID, HashSet<usize>>,
    rng: ThreadRng,
}

impl SystemService for WSServer {}
impl Supervised for WSServer {}

impl WSServer {
    /// Send message to all clients for an instance
    fn send_message(&self, instance: &ID, message: &str, skip_id: usize) {
        if let Some(sessions) = self.instances.get(instance) {
            for id in sessions {
                debug!("Evaluating to {}", id);
                if *id != skip_id {
                    debug!("Sending to {}", id);
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }

    /// Send global message to all clients
    fn send_global_message(&self, message: &str) {
        for (id, client) in self.sessions.iter() {
            debug!("Sending to {:?}", id);
            let _ = client.do_send(Message(message.to_owned()));
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
    pub addr: Recipient<Message>,
}

impl Handler<Connect> for WSServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("Someone joined");

        // notify all users in same room
        self.send_message(&1, "Someone joined", 0);

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
        self.send_global_message("Instance created");
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
        // send message to other users
        for instance in instances {
            self.send_message(&instance, "Someone disconnected", 0);
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
        // send message to other users
        for instance in instances {
            self.send_message(&instance, "Someone disconnected", 0);
        }

        if self.instances.get_mut(&instance).is_none() {
            //TODO: handle invalid instance
            // self.instances.insert(name.clone(), HashSet::new());
        }
        self.send_message(&instance, "Someone connected", id);
        self.instances.get_mut(&instance).unwrap().insert(id);
    }
}
