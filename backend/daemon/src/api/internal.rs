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
use tokio::{net::TcpListener, runtime};
use tower_web::middleware::log::LogMiddleware;
use tower_web::*;

use std::net::SocketAddr;

use super::{get_instance_by_id, APIErr};
use daemon::Instances;
use instance::InstanceType;
use models::{DefaultResponse, HeartbeatReq, InstanceStartedReq};
use SETTINGS;

/// Address parser for internal
/// Used also for runtime checks
pub fn parse_addr() -> Fallible<SocketAddr> {
    super::parse_address(
        &SETTINGS.main.api_internal_bind_ip,
        &SETTINGS.main.api_internal_bind_port,
    )
}

/// Start api server
pub fn start_server(runtime: &mut runtime::Runtime, instances: Instances) -> Fallible<()> {
    let addr = parse_addr()?;
    let incoming = TcpListener::bind(&addr)
        .map_err(|e| APIErr::BindError(e))?
        .incoming();

    runtime.spawn(
        ServiceBuilder::new()
            .middleware(LogMiddleware::new("yamba_backend::api::internal"))
            .resource(InternalAPI { instances })
            .serve(incoming),
    );
    Ok(())
}

struct InternalAPI {
    instances: Instances,
}

impl_web! {
    impl InternalAPI {

        #[post("/internal/started")]
        #[content_type("application/json")]
        fn connected(&self, body: InstanceStartedReq) -> Fallible<DefaultResponse> {
            debug!("instance started request: {:?}",body);
            let success = match get_instance_by_id(&self.instances, &body.id) {
                Some(v) => {
                    match v.voip {
                        InstanceType::Teamspeak(ref ts) => ts.on_connected(body.pid)?,
                    }
                    true
                }
                None => false,

            };
            Ok(DefaultResponse{success,msg: None})
        }

        #[post("/internal/heartbeat")]
        #[content_type("application/json")]
        fn heartbeat(&self, body: HeartbeatReq) -> Fallible<DefaultResponse> {
            let success = match get_instance_by_id(&self.instances, &body.id) {
                Some(v) => {
                    //TODO
                true
                }
                None => false,

            };
            Ok(DefaultResponse{success,msg: None})
        }
    }
}
