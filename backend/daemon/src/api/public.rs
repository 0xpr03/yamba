/*
 *  This file is part of yamba.
 *
 *  yamba is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  yamba is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with yamba.  If not, see <https://www.gnu.org/licenses/>.
 */

use failure::Fallible;
use http_r::{response::Response, status::StatusCode};
use tokio::{net::TcpListener, runtime};
use tower_web::view::Handlebars;
use tower_web::*;
use yamba_types::models::callback::ResolveResponse;
use yamba_types::models::*;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::callback::send_resolve;
use super::*;
use daemon::{create_instance, instance, InstanceBase, Instances};
use ytdl_worker::{RSongs, YTRequest};
use SETTINGS;

static CALLBACK_TICKET: AtomicUsize = AtomicUsize::new(0);

/// API server

/// Address parser for public
/// Used also for runtime checks
pub fn parse_addr() -> Fallible<SocketAddr> {
    super::parse_address(&SETTINGS.main.api_bind_ip, &SETTINGS.main.api_bind_port)
}

/// Start api server
pub fn start_server(
    runtime: &mut runtime::Runtime,
    instances: Instances,
    base: InstanceBase,
) -> Fallible<()> {
    let addr = parse_addr()?;
    let incoming = TcpListener::bind(&addr)
        .map_err(|e| APIErr::BindError(e))?
        .incoming();

    runtime.spawn(
        ServiceBuilder::new()
            .resource(ApiResource { instances, base })
            .serializer(Handlebars::new())
            .serve(incoming),
    );
    Ok(())
}

struct ApiResource {
    instances: Instances,
    base: InstanceBase,
}

unsafe impl Send for ApiResource {}
unsafe impl Sync for ApiResource {}

/// Struct for resolve jobs
struct ResolveDispatcher {
    url: String,
    ticket: Ticket,
}

impl ResolveDispatcher {
    pub fn new(req: ResolveRequest, ticket: usize) -> ResolveDispatcher {
        ResolveDispatcher {
            ticket,
            url: req.url,
        }
    }
}

impl YTRequest for ResolveDispatcher {
    fn url(&self) -> &str {
        &self.url
    }

    fn callback(&mut self, songs: RSongs, _: Instances) {
        let response = match songs {
            Ok(s) => ResolveResponse {
                ticket: self.ticket,
                success: true,
                songs: s,
                msg: None,
            },
            Err(e) => ResolveResponse {
                ticket: self.ticket,
                success: false,
                songs: Vec::new(),
                msg: Some(format!("{}", e)),
            },
        };

        send_resolve(&response);
    }
}

impl_web! {
    impl ApiResource {

        #[get("/")]
        #[content_type("html")]
        #[web(template = "web")]
        fn html(&self) -> Fallible<InstanceOverviewResponse> {
            let inst_r = self.instances.read().expect("Can't write instances!");
            let instances = inst_r.values().map(|r|{
                InstanceOverview {
                    id: r.get_id(),
                    playing: r.is_playing(),
                    volume: r.get_volume(),
                    inst_type: match r.get_voip() {
                        instance::InstanceType::Teamspeak(_) => String::from("Teamspeak"),
                    },
                    playback_info: r.playback_info(),
                }
            }).collect();
            Ok(InstanceOverviewResponse{instances})
        }

        #[get("/resolve/url")]
        #[content_type("application/json")]
        fn resolve(&self, query_string: ResolveRequest) -> Rsp {
            debug!("url resolve request: {:?}",query_string);
            match get_instance_by_id(&self.instances, &query_string.instance) {
                Some(v) => {
                    let t = CALLBACK_TICKET.fetch_add(1, Ordering::SeqCst);
                    let dispatcher = ResolveDispatcher::new(query_string, t.clone());
                    match v.dispatch_resolve(dispatcher.wrap()) {
                        Ok(_) => ok_response(ResolveTicketResponse{ticket: t}),
                        Err(_) => custom_response(StatusCode::TOO_MANY_REQUESTS,ErrorResponse{msg: String::from("Queue overload!"),details: ErrorCodes::RESOLVE_QUEUE_OVERLOAD})
                    }
                }
                None => invalid_instance()
            }
        }

        #[post("/instance/start")]
        #[content_type("application/json")]
        fn instance_start(&self, body: InstanceLoadReq) -> Rsp {
            debug!("instance start request: {:?}",body);
            let mut inst_w = self.instances.write().expect("Can't write instances!");
            if !inst_w.contains_key(&body.id) {
                create_instance(&self.base, body).map(|v|  {inst_w.insert(v.get_id(),v); ok() })?
            } else {
                custom_response(StatusCode::CONFLICT,ErrorResponse{msg: String::from("Instance running!"),details: ErrorCodes::INSTANCE_RUNNING})
            }
        }

        #[get("/instance/list")]
        #[content_type("application/json")]
        fn instance_list(&self) -> Fallible<InstanceListResponse> {
            debug!("instance list request");
            let inst_r = self.instances.read().expect("Can't write instances!");
            let ids = inst_r.keys().map(|v|v.clone()).collect();
            Ok(InstanceListResponse{instances:ids})
        }

        #[post("/instance/stop")]
        #[content_type("application/json")]
        fn instance_stop(&self, body: InstanceStopReq) -> Rsp {
            debug!("instance stop request: {:?}",body);
            let mut inst_w = self.instances.write().expect("Can't write instances!");
            match inst_w.remove(&body.id) {
                Some(_) => ok(),
                None => invalid_instance(),
            }
        }


        #[post("/playback/url")]
        #[content_type("application/json")]
        fn playback_start(&self, body: PlaybackUrlReq) -> Rsp {
            // if body.song.source TODO: check for non-localhost URL
            debug!("playback request: {:?}",body);
            match get_instance_by_id(&self.instances, &body.id) {
                Some(v) => {v.play_track(body.song)?; ok() },
                None => invalid_instance(),
            }
        }

        #[post("/playback/pause")]
        #[content_type("application/json")]
        fn playback_pause(&self, body: PlaybackPauseReq) -> Rsp {
            debug!("playback pause request: {:?}",body);
            match get_instance_by_id(&self.instances, &body.id) {
                Some(v) =>  {v.pause(); ok()},
                None => invalid_instance(),
            }
        }

        #[get("/playback/state")]
        #[content_type("application/json")]
        fn playback_state(&self, query_string: StateGetReq) -> Rsp {
            debug!("playback state request: {:?}",query_string);
            custom_response(StatusCode::NOT_IMPLEMENTED,ErrorResponse{details: ErrorCodes::NONE,msg: String::from("not implemented yet")})
        }

        #[post("/volume")]
        #[content_type("application/json")]
        fn volume_set(&self, body: VolumeSetReq) -> Rsp {
            trace!("volume set: {:?}", body);
            match get_instance_by_id(&self.instances, &body.id) {
                Some(inst) => {
                    inst.set_volume(body.volume);
                    ok()
                }
                None => invalid_instance(),
            }
        }

        #[get("/volume")]
        #[content_type("application/json")]
        fn volume_get(&self, query_string: VolumeGetReq) -> Fallible<VolumeResponse> {
            trace!("Volume get: {:?}",query_string);
            let volume = get_instance_by_id(&self.instances, &query_string.id).map(|inst| inst.get_volume());
            Ok(VolumeResponse{volume,msg: None})
        }
    }
}
