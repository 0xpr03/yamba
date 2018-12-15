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
use futures::{future, Future, Stream};
use gst;
use hashbrown::HashMap;
use hyper::{self, Body, Response};
use mysql::Pool;
use tokio::{self, runtime::Runtime};
use tokio_signal::unix::{self, Signal};

use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use api;
use audio::{self, CContext, CMainloop, NullSink};
use db;
use models::SongMin;
use playback::{self, PlaybackSender, Player, PlayerEvent};
use rpc;
use ts::TSInstance;
use ytdl::YtDL;
use ytdl_worker;

use SETTINGS;

/// Base for each instance
pub struct Instance {
    pub id: ID,
    pub voip: InstanceType,
    pub current_song: RwLock<Option<SongMin>>,
    pub player: Player,
    pub db: Pool,
}

/// Instance type for different VoIP systems
pub enum InstanceType {
    Teamspeak(Teamspeak),
}

/// Teamspeak specific VoIP instance
pub struct Teamspeak {
    ts: TSInstance,
    sink: NullSink,
    updated: RwLock<Instant>,
}

impl Teamspeak {
    /// Setup call on successfull connection
    pub fn on_connected(&self) {
        self.sink.set_monitor_for_process(self.ts.get_process_id());
    }
}

/// Daemon init & startup of all servers

// types used by rpc, api, playback daemons
pub type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
pub type APIChannel = mpsc::Sender<api::APIRequest>;
pub type Instances<'a> = Arc<RwLock<HashMap<i32, Instance>>>;
pub type ID = Arc<i32>;

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
            )
            .into())
        }
    };

    let (tx, rx) = mpsc::channel::<api::APIRequest>(100);
    let (player_tx, player_rx) = mpsc::channel::<PlayerEvent>(10);

    let (mainloop, context) = audio::init()?;

    audio::unload_problematic_modules(&mainloop, &context)?;

    // sink to avoid errors due to no sink existing & avoid glitches
    let default_sink = NullSink::new(mainloop.clone(), context.clone(), "default_sink")?;
    default_sink.mute_sink(true)?;
    default_sink.set_source_as_default()?;
    default_sink.set_sink_as_default()?;

    let mut rt = Runtime::new().map_err(|e| DaemonErr::RuntimeCreationError(e))?;

    rpc::create_rpc_server(&mut rt, instances.clone())
        .map_err(|e| DaemonErr::RPCCreationError(e))?;
    api::create_api_server(&mut rt, tx.clone()).map_err(|e| DaemonErr::APICreationError(e))?;
    playback::create_playback_server(&mut rt, player_rx, pool.clone())?;
    ytdl_worker::create_ytdl_worker(&mut rt, rx, ytdl.clone(), pool.clone());

    info!("Loading instances..");

    match load_instances(&instances, pool.clone(), player_tx, &mainloop, &context) {
        Ok(_) => (),
        Err(e) => {
            error!("Unable to load instances: {}", e);
            return Err(DaemonErr::InitializationError(format!("{}", e)).into());
        }
    }

    info!("Daemon initialized");
    let ft_sigint = Signal::new(unix::libc::SIGINT)
        .flatten_stream()
        .into_future();
    let ft_sigterm = Signal::new(unix::libc::SIGTERM)
        .flatten_stream()
        .into_future();
    let ftb_sigquit = Signal::new(unix::libc::SIGQUIT)
        .flatten_stream()
        .into_future();
    match rt.block_on(future::select_all(vec![ft_sigint, ft_sigterm, ftb_sigquit])) {
        Err(e) => {
            let ((err, _), _, _) = e;
            info!("Shutting down daemon..");
            println!("Shutting down daemon..");
            return Err(DaemonErr::ShutdownError(err).into());
        }
        Ok(_) => (),
    };
    info!("Daemon stopped");
    println!("Daemon stopped");
    Ok(())
}

/// Load instances
/// Stops previous instances
fn load_instances(
    instances: &Instances,
    pool: Pool,
    player_send: PlaybackSender,
    mainloop: &CMainloop,
    context: &CContext,
) -> Fallible<()> {
    let mut instances = instances.write().expect("Main RwLock is poisoned!");
    instances.clear();
    let instance_ids = db::get_autostart_instance_ids(&pool)?;
    for id in instance_ids {
        let instance =
            match create_instance_from_id(&id, &pool, player_send.clone(), &mainloop, &context) {
                Ok(v) => v,
                Err(e) => {
                    error!("Unable to load instance ID {}: {}", id, e);
                    continue;
                }
            };
        instances.insert(id, instance);
    }
    Ok(())
}

/// Load & create single instance by ID
fn create_instance_from_id(
    id: &i32,
    pool: &Pool,
    player_send: PlaybackSender,
    mainloop: &CMainloop,
    context: &CContext,
) -> Fallible<Instance> {
    let data = db::load_instance_data(&pool, id)?;

    let id = Arc::new(data.id);

    Ok(Instance {
        voip: InstanceType::Teamspeak(Teamspeak {
            ts: TSInstance::spawn(&data, &SETTINGS.main.rpc_bind_port)?,
            sink: NullSink::new(mainloop.clone(), context.clone(), format!("sink{}", &id))?,
            updated: RwLock::new(Instant::now()),
        }),
        player: Player::new(player_send, id.clone())?,
        id: id,
        current_song: RwLock::new(None),
        db: pool.clone(),
    })
}

/// Parse socket address
pub fn parse_socket_address(ip: &str, port: u16) -> Fallible<SocketAddr> {
    Ok(SocketAddr::new(ip.parse()?, port))
}
