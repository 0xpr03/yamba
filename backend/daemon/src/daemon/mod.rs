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

mod heartbeat;
pub mod instance;

pub use self::heartbeat::{HeartBeatInstance, HeartbeatMap};
use failure::Fallible;
use futures::sync::mpsc;
use futures::{future, Future, Stream};
use gstreamer as gst;
use hashbrown::HashMap;
use tokio::{self, runtime::Runtime};
use tokio_signal::unix::{self, Signal};

use std::env::{args, current_exe};
use std::i32;
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc, RwLock, Weak};
use std::thread;

use self::instance::*;
use crate::api;
use crate::audio::{self, CContext, CMainloop, NullSink};
use crate::cache::Cache;
use crate::playback::{PlaybackSender, Player, PlayerEvent};
use crate::ts::TSInstance;
use crate::ytdl::YtDL;
use crate::ytdl_worker;
use crate::SETTINGS;
use yamba_types::models::{self, SongID, TSSettings};

/// Daemon init & startup of all servers

// types used by rpc, api, playback daemons
// pub type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;
pub type Instances = Arc<RwLock<HashMap<i32, Instance>>>;
pub type WInstances = Weak<RwLock<HashMap<i32, Instance>>>;

#[derive(Fail, Debug)]
pub enum DaemonErr {
    #[fail(display = "Unable to open default config {}", _0)]
    RuntimeCreationError(#[cause] tokio::io::Error),
    #[fail(display = "Unable initialize daemon {}", _0)]
    InitializationError(String),
}

/// Base for creating instances
pub struct InstanceBase {
    pub player_send: PlaybackSender,
    pub mainloop: CMainloop,
    pub context: CContext,
    pub default_sink: Arc<NullSink>,
    pub ytdl: Arc<YtDL>,
    pub cache: SongCache,
    pub controller: ytdl_worker::Controller,
    pub w_instances: WInstances,
    pub heartbeat: HeartbeatMap,
}

impl InstanceDataProvider for InstanceBase {
    fn get_controller(&self) -> &ytdl_worker::Controller {
        &self.controller
    }
    fn get_ytdl(&self) -> &Arc<YtDL> {
        &self.ytdl
    }
    fn get_cache(&self) -> &SongCache {
        &self.cache
    }
    fn get_weak_instances(&self) -> &WInstances {
        &self.w_instances
    }
}

unsafe impl Send for InstanceBase {}
unsafe impl Sync for InstanceBase {}

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

        info!("Performing ytdl startup check..");
        match ytdl.startup_test() {
            true => debug!("Startup check success"),
            false => {
                return Err(DaemonErr::InitializationError(
                    "Startup check failed for ytdl engine!".into(),
                )
                .into());
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

        let controller = ytdl_worker::crate_ytdl_scheduler(
            &mut rt,
            ytdl.clone(),
            cache.clone(),
            instances.clone(),
        );

        ytdl_worker::crate_yt_updater(&mut rt, ytdl.clone());

        create_playback_event_handler(&mut rt, player_rx, instances.clone())?;

        let base = InstanceBase {
            player_send: player_tx,
            mainloop: mainloop,
            context: context,
            default_sink: default_sink,
            ytdl: ytdl,
            cache: cache,
            controller: controller,
            w_instances: Arc::downgrade(&instances),
            heartbeat: heartbeat::HeartbeatMap::new(instances.clone(), &mut rt),
        };

        api::start_server(&mut rt, instances.clone(), base)?;

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

/// Create instance
pub fn create_instance(base: &InstanceBase, inst: models::InstanceLoadReq) -> Fallible<Instance> {
    let inst = match inst.data {
        models::InstanceType::TS(settings) => {
            create_ts_instance(base, settings, inst.id, inst.volume)?
        }
    };

    let _ = api::callback::send_instance_state(&models::callback::InstanceStateResponse {
        id: inst.get_id(),
        state: models::callback::InstanceState::Started,
    })
    .map_err(|e| warn!("Can't send instance started {}", e));

    Ok(inst)
}

/// Crate instance of type TS
fn create_ts_instance(
    base: &InstanceBase,
    data: TSSettings,
    id: ID,
    volume: f64,
) -> Fallible<Instance> {
    let player = Player::new(base.player_send.clone(), id.clone(), volume)?;
    let sink = NullSink::new(
        base.mainloop.clone(),
        base.context.clone(),
        format!("yambasink{}", &id),
    )?;
    player.set_pulse_device(sink.get_sink_name())?;
    let voip = InstanceType::Teamspeak(Teamspeak {
        ts: TSInstance::spawn(
            &data,
            &id,
            &SETTINGS.main.api_internal_bind_ip,
            &SETTINGS.main.api_internal_bind_port,
            &SETTINGS.main.api_jsonrpc_ip,
            &SETTINGS.main.api_jsonrpc_port,
        )?,
        sink,
        mute_sink: base.default_sink.clone(),
    });

    Ok(Instance::new(
        id,
        voip,
        base,
        player,
        base.heartbeat.clone(),
    ))
}
