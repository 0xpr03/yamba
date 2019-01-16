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

use arraydeque::ArrayDeque;
use failure::{self, Fallible};
use futures::sync::mpsc;
use futures::{future, Future, Stream};
use gst;
use hashbrown::HashMap;
use hyper::{self, Body, Response};
use mysql::Pool;
use tokio::{self, runtime::Runtime};
use tokio_signal::unix::{self, Signal};

use std::env::{args, current_exe};
use std::i32;
use std::net::SocketAddr;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, Mutex, RwLock};
use std::thread;
use std::time::Instant;

use api;
use audio::{self, CContext, CMainloop, NullSink};
use cache::Cache;
use db;
use instance::*;
use models::{DBInstanceType, InstanceStorage, SongID, TSSettings};
use playback::{self, PlaybackSender, Player, PlayerEvent};
use rpc;
use ts::TSInstance;
use ytdl::YtDL;
use ytdl_worker;
use SETTINGS;

/// Daemon init & startup of all servers

// types used by rpc, api, playback daemons
pub type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
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
}

/*/// Format player name
/// Standardizes the naming required for identification
fn format_player_name(id: &i32) -> String {
    format!("player#{}", id)
}*/

struct InstanceBase<'a> {
    pub pool: Pool,
    pub player_send: PlaybackSender,
    pub mainloop: &'a CMainloop,
    pub context: &'a CContext,
    pub default_sink: &'a Arc<NullSink>,
    pub ytdl: &'a Arc<YtDL>,
    pub cache: &'a SongCache,
    pub controller: &'a ytdl_worker::Controller,
}

/// Start runtime
pub fn start_runtime() -> Fallible<()> {
    info!("Starting daemon..");
    let sighub = Arc::new(AtomicBool::new(false));
    {
        gst::init()?;
        let glib_loop = glib::MainLoop::new(None, false);
        let glib_loop_clone = glib_loop.clone();
        thread::spawn(move || {
            let glib_loop = &glib_loop_clone;
            glib_loop.run();
        });
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

        let (player_tx, player_rx) = mpsc::channel::<PlayerEvent>(128);

        let (mainloop, context) = audio::init()?;

        audio::unload_problematic_modules(&mainloop, &context)?;

        // sink to avoid errors due to no sink existing & avoid glitches
        let default_sink = NullSink::new(mainloop.clone(), context.clone(), "default_sink")?;
        default_sink.mute_sink(true)?;
        default_sink.set_source_as_default()?;
        default_sink.set_sink_as_default()?;

        let default_sink = Arc::new(default_sink);

        let mut rt = Runtime::new().map_err(|e| DaemonErr::RuntimeCreationError(e))?;

        let cache = Cache::<SongID, String>::new(&mut rt);

        let controller = Box::new(ytdl_worker::crate_ytdl_scheduler(
            &mut rt,
            ytdl.clone(),
            pool.clone(),
            cache.clone(),
        ));

        rpc::create_rpc_server(&mut rt, instances.clone())
            .map_err(|e| DaemonErr::RPCCreationError(e))?;

        ytdl_worker::crate_yt_updater(&mut rt, ytdl.clone());

        // let api run on i32::max, as no instance will reach this
        let id_api = Arc::new(i32::MAX);
        api::create_api_server(&mut rt, controller.channel(id_api.clone(), 32))
            .map_err(|e| DaemonErr::APICreationError(e))?;
        playback::create_playback_server(&mut rt, player_rx, instances.clone())?;

        info!("Loading instances..");

        let inst_data = InstanceBase {
            pool: pool.clone(),
            player_send: player_tx,
            mainloop: &mainloop,
            context: &context,
            default_sink: &default_sink,
            ytdl: &ytdl,
            cache: &cache,
            controller: &controller,
        };

        match load_instances(inst_data, &instances) {
            Ok(_) => (),
            Err(e) => {
                error!("Unable to load instances: {}\n{}", e, e.backtrace());
                return Err(DaemonErr::InitializationError(format!("{}", e)).into());
            }
        }

        info!("Daemon initialized");

        let sighub_c = sighub.clone();
        rt.spawn(
            Signal::new(unix::SIGHUP)
                .flatten_stream()
                .for_each(move |_| {
                    debug!("Sighub received");
                    sighub_c.store(true, Ordering::Relaxed);
                    Ok(())
                })
                .map_err(|e| error!("sighub error: {}", e)),
        );

        let ft_sigint = Signal::new(unix::SIGINT).flatten_stream().into_future();
        let ft_sigterm = Signal::new(unix::SIGTERM).flatten_stream().into_future();
        let ftb_sigquit = Signal::new(unix::SIGQUIT).flatten_stream().into_future();
        let ftb_sighub = Signal::new(unix::SIGHUP).flatten_stream().into_future();
        match rt.block_on(future::select_all(vec![
            ft_sigint,
            ft_sigterm,
            ftb_sigquit,
            ftb_sighub,
        ])) {
            Err(e) => {
                // first tuple element conains error, but is neither display nor debug..
                let ((_, _), _, _) = e;
                error!("Error in signal handler");
                println!("Shutting down daemon..");
            }
            Ok(_) => (),
        };
        drop(rt);
        drop(instances);
        glib_loop.quit();
        drop(pool);
        info!("Daemon stopped");
        println!("Daemon stopped");
    }
    if sighub.load(Ordering::Relaxed) {
        info!("Detected sighub, restarting..");
        restart()
    }

    Ok(())
}

fn restart() {
    // first element is exec itself, remove it
    let args_origin = args().enumerate().filter(|&(i, _)| i > 0).map(|(_, e)| e);
    let args: Vec<_> = args_origin.collect();
    println!(
        "Failed restarting: {}",
        Command::new(current_exe().unwrap()).args(args).exec()
    );
}

/// Load instances
/// Stops previous instances
fn load_instances(base: InstanceBase, instances: &Instances) -> Fallible<()> {
    let mut instances = instances.write().expect("Main RwLock is poisoned!");
    instances.clear();
    let instance_ids = db::get_autostart_instance_ids(&base.pool)?;
    for id in instance_ids {
        let instance = match create_instance_from_id(&base, &id) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    "Unable to load instance ID {}: {}\n{}",
                    id,
                    e,
                    e.backtrace()
                );
                continue;
            }
        };
        instances.insert(id, instance);
    }
    Ok(())
}

/// Load & create single instance by ID
fn create_instance_from_id(base: &InstanceBase, id: &i32) -> Fallible<Instance> {
    let data = db::load_instance_data(&base.pool, id)?;
    let storage = db::read_instance_storage(id, &base.pool)?;

    // can't match untill we have more than one type
    let DBInstanceType::TS(ts_data) = data;
    create_ts_instance(base, ts_data, storage)
}

/// Crate instance of type TS
fn create_ts_instance(
    base: &InstanceBase,
    data: TSSettings,
    storage: InstanceStorage,
) -> Fallible<Instance> {
    let id = Arc::new(data.id);

    let player = Player::new(base.player_send.clone(), id.clone(), storage.volume)?;
    let sink = NullSink::new(
        base.mainloop.clone(),
        base.context.clone(),
        format!("yambasink{}", &id),
    )?;

    player.set_pulse_device(sink.get_sink_name())?;
    Ok(Instance {
        voip: Arc::new(InstanceType::Teamspeak(Teamspeak {
            ts: TSInstance::spawn(&data, &SETTINGS.main.rpc_bind_port)?,
            sink,
            mute_sink: base.default_sink.clone(),
            updated: RwLock::new(Instant::now()),
        })),
        player: Arc::new(player),
        ytdl_tx: base
            .controller
            .channel(id.clone(), SETTINGS.ytdl.instance_backlog_max as usize),
        id: id,
        playback_history: Arc::new(Mutex::new(ArrayDeque::new())),
        stop_flag: Arc::new(AtomicBool::new(false)),
        store: Arc::new(RwLock::new(storage)),
        pool: base.pool.clone(),
        ytdl: base.ytdl.clone(),
        cache: base.cache.clone(),
        current_song: Arc::new(RwLock::new(None)),
    })
}

/// Parse socket address
pub fn parse_socket_address(ip: &str, port: u16) -> Fallible<SocketAddr> {
    Ok(SocketAddr::new(ip.parse()?, port))
}
