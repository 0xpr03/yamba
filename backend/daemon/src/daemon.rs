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

use failure::{self, Fallible};
use futures::sync::mpsc;
use futures::{Future, Sink, Stream};
use gst;
use hashbrown::HashMap;
use hyper::{self, Body, Response};
use mysql::Pool;
use tokio::{self, runtime::Runtime};
use tokio_signal::unix::{self, Signal};
use tokio_threadpool::blocking;

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};

use api;
use db;
use models::{Queue, SongMin, TSSettings};
use playback::{PlaybackSender, Player};
use rpc;
use ts::TSInstance;
use ytdl::YtDL;
use ytdl_worker;

use SETTINGS;

pub struct Instance {
    id: i32,
    ts_instance: TSInstance,
    current_song: RwLock<Option<SongMin>>,
    player: Player,
}

/// Daemon init & startup of all servers

// type used by rpc & api
pub type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
pub type APIChannel = mpsc::Sender<api::APIRequest>;
pub type Instances<'a> = Arc<RwLock<HashMap<i32, Instance>>>;

#[derive(Fail, Debug)]
pub enum DaemonErr {
    #[fail(display = "Unable to open default config {}", _0)]
    RuntimeCreationError(#[cause] tokio::io::Error),
    #[fail(display = "Unable initialize daemon {}", _0)]
    InitializationError(String),
    #[fail(display = "Unable to create rpc server {}", _0)]
    RPCCreationError(#[cause] failure::Error),
    #[fail(display = "Unable to create api server {}", _0)]
    APICreationError(#[cause] failure::Error),
    #[fail(display = "Error on shutdown of runtime")]
    ShutdownError(#[cause] io::Error),
}

/// Format player name
/// Standardizes the naming required for identification
fn format_player_name(id: &i32) -> String {
    format!("player#{}", id)
}

/// Start runtime
pub fn start_runtime() -> Fallible<()> {
    info!("Starting daemon..");
    gst::init()?;
    let instances: Instances = Arc::new(RwLock::new(HashMap::new()));
    let ytdl = Arc::new(YtDL::new()?);
    let pool = db::init_pool_timeout()?;

    info!("Performing ytdl startup check..");
    match ytdl.startup_test() {
        true => debug!("Startup check success"),
        false => {
            return Err(DaemonErr::InitializationError(
                "Startup check failed for ytdl engine!".into(),
            ).into())
        }
    };

    let (tx, rx) = mpsc::channel::<api::APIRequest>(100);

    let mut rt = Runtime::new().map_err(|e| DaemonErr::RuntimeCreationError(e))?;

    rpc::create_rpc_server(&mut rt).map_err(|e| DaemonErr::RPCCreationError(e))?;
    api::create_api_server(&mut rt, tx.clone()).map_err(|e| DaemonErr::APICreationError(e))?;
    ytdl_worker::create_ytdl_worker(&mut rt, rx, ytdl.clone(), pool.clone());

    info!("Daemon initialized");
    match rt.block_on(
        Signal::new(unix::libc::SIGINT)
            .flatten_stream()
            .into_future()
            .select(
                Signal::new(unix::libc::SIGTERM)
                    .flatten_stream()
                    .into_future(),
            ),
    ) {
        Err(e) => {
            let ((err, _), _) = e;
            return Err(DaemonErr::ShutdownError(err).into());
        }
        Ok(_) => (),
    };
    info!("Daemon stopped");
    Ok(())
}

/// Load instances
/// Stops previous instances
fn load_instances(instances: Instances, pool: Pool) -> Fallible<()> {
    let mut instances = instances.write().expect("Main RwLock poisoned!");
    instances.clear();
    let instance_ids = db::get_instance_ids(&pool)?;
    for id in instance_ids {
        let data = db::load_instance_data(&pool, id)?;

        //instances.insert(id, instance);
    }
    Ok(())
}

/// Load & create single instance by ID
fn create_instance_from_id(
    id: &i32,
    pool: &Pool,
    player_send: PlaybackSender,
) -> Fallible<Instance> {
    let data = db::load_instance_data(&pool, id)?;

    Ok(Instance {
        ts_instance: TSInstance::spawn(&data, &SETTINGS.main.rpc_bind_port)?,
        id: data.id,
        player: Player::new(player_send, &format_player_name(id))?,
        current_song: RwLock::new(None),
    })
}

/// Parse socket address
pub fn parse_socket_address(ip: &str, port: u16) -> Fallible<SocketAddr> {
    Ok(SocketAddr::new(ip.parse()?, port))
}
