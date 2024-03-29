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

use chrono::offset::Utc;
use failure::Fallible;
use futures::sync::mpsc::Receiver;
use futures::Stream;
use gst::ResourceError;
use gst_player::PlayerError;
use tokio::runtime;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};
use std::thread;

use api::callback;
use audio::NullSink;
use cache::Cache;
use daemon::{HeartbeatMap, Instances, WInstances};
use playback::{PlaybackState, Player, PlayerEvent, PlayerEventType};
use ts::TSInstance;
use yamba_types::models::{callback::*, CacheSong, InstanceStartedReq, Song, SongID, TimeStarted};
use ytdl::YtDL;
use ytdl_worker::{Controller, YTReqWrapped, YTSender};
use SETTINGS;

/// module containing a single instance

const RETRY_MAX: usize = 3;

#[derive(Fail, Debug, PartialEq)]
pub enum InstanceErr {
    #[fail(display = "No Audio track for URL {}", _0)]
    NoAudioTrack(String),
    #[fail(display = "No YTDL result for URL {}", _0)]
    InvalidSource(String),
    #[fail(display = "No current song, when one was expected")]
    NoCurrentSong,
    #[fail(display = "Max amount of error retries reached for song")]
    MaxRetries,
}

/// Data provider for creation of instances
pub trait InstanceDataProvider {
    fn get_controller(&self) -> &Controller;
    fn get_ytdl(&self) -> &Arc<YtDL>;
    fn get_cache(&self) -> &SongCache;
    fn get_weak_instances(&self) -> &WInstances;
}

pub type ID = i32;
/// Cache for resolved media URIs
pub type SongCache = Cache<SongID, CacheSong>;
#[allow(non_camel_case_types)]
type CURRENT_SONG = Arc<RwLock<Option<CurrentSong>>>;

/// Type holding the current song
pub type CurrentSong = Song;

/// Base for each instance
pub struct Instance {
    id: ID,
    voip: InstanceType,
    player: Player,
    ytdl: Arc<YtDL>,
    current_song: CURRENT_SONG,
    error_retries: AtomicUsize,
    cache: SongCache,
    instances: WInstances,
    url_resolve: YTSender,
    startup_time: TimeStarted,
    state: RwLock<InstanceState>,
}

impl Drop for Instance {
    fn drop(&mut self) {
        // don't store on clone drop
        println!("Dropping instance {}", self.id);
        self.player.stop();

        let _ = callback::send_instance_state(&InstanceStateResponse {
            id: self.get_id(),
            state: InstanceState::Stopped,
        })
        .map_err(|e| warn!("Can't send instance stopped: {}", e));
    }
}

impl Instance {
    pub fn new(
        id: ID,
        voip: InstanceType,
        base: &InstanceDataProvider,
        player: Player,
        heartbeats: HeartbeatMap,
    ) -> Instance {
        let instance = Instance {
            voip: voip,
            url_resolve: base.get_controller().channel(id.clone(), 64),
            player,
            id: id,
            ytdl: base.get_ytdl().clone(),
            cache: base.get_cache().clone(),
            current_song: Arc::new(RwLock::new(None)),
            instances: base.get_weak_instances().clone(),
            error_retries: AtomicUsize::new(0),
            startup_time: Utc::now().timestamp(),
            state: RwLock::new(InstanceState::Started),
        };

        heartbeats.update(instance.get_id());

        instance
    }

    /// Reset error retry count for song
    fn reset_error_retries(&self) {
        self.error_retries.store(0, Ordering::Relaxed);
    }

    /// Increase amount of error retries for song
    fn increate_error_retries(&self) {
        self.error_retries.fetch_add(1, Ordering::SeqCst);
    }

    /// Get error retry count for song
    fn get_error_retries(&self) -> usize {
        self.error_retries.load(Ordering::Relaxed)
    }

    /// Resolve URL under this instances queue
    pub fn dispatch_resolve(&self, request: YTReqWrapped) -> Fallible<()> {
        Ok(self.url_resolve.try_send(request)?)
    }

    /// Stop playback
    pub fn stop_playback(&self) {
        let mut lock = self.current_song.write().expect("Can't lock current song!");
        *lock = None;
        // don't store to history, still in queue, no end-of-stream triggered
        self.player.stop();
    }

    /// Handle invalid cache entries
    /// Re-resolves and restarts playback
    fn force_song_retry(&self) -> Fallible<()> {
        let song_r = self.current_song.read().expect("Can't lock current song!");
        match *song_r {
            Some(ref v) => {
                if self.get_error_retries() >= RETRY_MAX {
                    return Err(InstanceErr::MaxRetries.into());
                }
                self.increate_error_retries();
                let source = v.source.clone();
                let songid = v.id.clone();
                self.cache.delete(&songid);
                let instances = self.instances.clone();
                let cache = self.cache.clone();
                let id = self.id.clone();
                let ytdl = self.ytdl.clone();
                thread::spawn(move || {
                    if let Err(e) =
                        Instance::play_track_inner(instances, cache, id, ytdl, source, songid, true)
                    {
                        warn!("Error while retrying track! {}", e);
                    }
                });
                Ok(())
            }
            None => Err(InstanceErr::NoCurrentSong.into()),
        }
    }

    /// Handle end of stream event
    fn end_of_stream(&self) {
        // !stop-flag && no current song (avoid feedback loop)
        let has_current_song = self
            .current_song
            .read()
            .expect("Can't lock current song!")
            .is_some();

        if has_current_song {
            self.send_playstate_change(Playstate::EndOfMedia);
        } else {
            trace!("Ignoring end of stream");
        }
    }

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn get_voip(&self) -> &InstanceType {
        &self.voip
    }

    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }

    pub fn set_volume(&self, v: f64) {
        self.player.set_volume(v);
        if let Err(e) = callback::send_volume_change(&VolumeChange {
            id: self.get_id(),
            volume: v,
        }) {
            info!("Couldn't send volume change {}", e);
        }
    }

    pub fn get_volume(&self) -> f64 {
        self.player.get_volume()
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn get_playback_state(&self) -> Playstate {
        playback_to_public_state(self.player.get_state())
    }

    pub fn play(&self) {
        self.player.play();
    }

    /// Returns startup time as UNIX timestamp
    pub fn get_startup_time(&self) -> i64 {
        self.startup_time
    }

    /// Resume playback
    pub fn resume_playback(&self) -> Fallible<()> {
        if self
            .current_song
            .read()
            .expect("can't lock current song")
            .is_some()
        {
            self.player.play();
        }
        Ok(())
    }

    /// Returns current instance state  
    /// Must never be stopped.
    pub fn get_state(&self) -> InstanceState {
        self.state.read().expect("Can't read state").clone()
    }

    /// Called when voip is connected & able to send audio
    pub(crate) fn connected(&self, param: InstanceStartedReq) -> Fallible<()> {
        match self.voip {
            InstanceType::Teamspeak(ref ts) => ts.on_connected(param.pid)?,
        }
        let mut state = self.state.write().expect("Can't lock state!");
        *state = InstanceState::Running;
        drop(state);

        let _ = callback::send_instance_state(&InstanceStateResponse {
            id: self.get_id(),
            state: InstanceState::Running,
        })
        .map_err(|e| warn!("Can't send instance running state {}", e));

        Ok(())
    }

    /// Get current playback info
    /// Contains track name, artist, current playback position, total length
    pub fn playback_info(&self) -> String {
        let song_guard = self.current_song.read().expect("Can't lock current track!");
        match song_guard.as_ref() {
            Some(cur_song) => {
                let position = self.player.get_position();
                let length = format_time(cur_song.length);
                let artist = match cur_song.artist.as_ref() {
                    Some(v) => format!(" - {}", v),
                    None => String::new(),
                };
                format!(
                    "{}{} {:02}:{:02} / {} {}",
                    cur_song.name.as_str(),
                    artist,
                    position.minutes + (position.hours * 60),
                    position.seconds,
                    length,
                    match self.player.is_paused() {
                        true => "-paused-",
                        false => "",
                    }
                )
            }
            None => String::from("Playback ended"),
        }
    }

    /// Play song
    pub fn play_track(&self, song: Song) -> Fallible<()> {
        let mut c_song_w = self.current_song.write().expect("Can't lock current song!");

        let source = song.source.clone();
        let songid = song.id.clone();
        *c_song_w = Some(song);
        let instances = self.instances.clone();
        let cache = self.cache.clone();
        let id = self.id.clone();
        let ytdl = self.ytdl.clone();
        thread::spawn(move || {
            if let Err(e) =
                Instance::play_track_inner(instances, cache, id, ytdl, source, songid, false)
            {
                warn!("Error while resolving next track! {}", e);
            }
        });

        Ok(())
    }

    /// Send playstate change
    fn send_playstate_change(&self, state: Playstate) {
        if let Err(e) = callback::send_playback_state(&PlaystateResponse {
            id: self.id.clone(),
            state,
        }) {
            error!("Can't send playback state change: {}", e);
        }
    }

    /// Send position change
    fn send_position_update(id: ID, position_ms: u32) {
        if let Err(e) =
            callback::send_track_position_update(&TrackPositionUpdate { id, position_ms })
        {
            error!("Can't send playback state change: {}", e);
        }
    }

    /// Inner function, blocking
    /// Resolves the playback URI
    fn play_track_inner(
        instances: WInstances,
        cache: SongCache,
        id: ID,
        ytdl: Arc<YtDL>,
        source: String,
        song_id: SongID,
        retry: bool,
    ) -> Fallible<()> {
        let audio_url: String = if let Some(v) = cache.get(&song_id) {
            v
        } else {
            debug!("No cache entry for {}", song_id);
            let track = ytdl.get_url_info(source.as_str())?;
            let track = match track.get(0) {
                Some(t) => t,
                None => return Err(InstanceErr::InvalidSource(source).into()),
            };

            let url_temp = match track.best_audio_format(SETTINGS.ytdl.min_audio_bitrate) {
                Some(v) => v.url.clone(),
                None => return Err(InstanceErr::NoAudioTrack(source).into()),
            };
            cache.upsert(song_id, url_temp.clone());
            url_temp
        };

        let instances = match instances.upgrade() {
            Some(v) => v,
            None => return Ok(()),
        };
        let lock = instances.read().expect("Can't read instances!");
        if let Some(inst) = lock.get(&id) {
            if !retry {
                inst.reset_error_retries();
            }
            inst.player.set_uri(audio_url.as_str());
        } else {
            warn!("Instance gone, ignoring playback resolver..");
        }

        Ok(())
    }
}

/// Format time by
#[inline(always)]
fn format_time(length: Option<u32>) -> String {
    match length {
        Some(v) => format!("{:02}:{:02}", v / 60, v % 60),
        None => String::from("--:--"),
    }
}

/// Instance type for different VoIP systems
pub enum InstanceType {
    Teamspeak(Teamspeak),
}

/// Teamspeak specific VoIP instance
pub struct Teamspeak {
    pub ts: TSInstance,
    pub sink: NullSink,
    pub mute_sink: Arc<NullSink>,
}

impl Teamspeak {
    /// Setup call on successfull connection
    /// process_id is the real ts id, as the xvfb wrapper doesn't count
    pub fn on_connected(&self, process_id: u32) -> Fallible<()> {
        trace!("Setting monitor for ts");
        self.sink.set_monitor_for_process(process_id)?;
        trace!("Setting sink for ts");
        self.mute_sink.set_sink_for_process(process_id)?;
        Ok(())
    }
}

/// Convert PlaybackState from Player to PlayerState from Types
#[inline(always)]
fn playback_to_public_state(state: PlaybackState) -> Playstate {
    match state {
        PlaybackState::Playing => Playstate::Playing,
        PlaybackState::Stopped => Playstate::Stopped,
        PlaybackState::Paused => Playstate::Paused,
    }
}

/// Register event handler for playback in daemon
pub fn create_playback_event_handler(
    runtime: &mut runtime::Runtime,
    tx: Receiver<PlayerEvent>,
    instances: Instances,
) -> Fallible<()> {
    let stream = tx.for_each(move |event| {
        match event.event_type {
            PlayerEventType::UriLoaded => {
                trace!("URI loaded for {}", event.id);
                let instances_r = instances.read().expect("Can't read instance!");
                if let Some(v) = instances_r.get(&event.id) {
                    v.play();
                }
            }
            PlayerEventType::VolumeChanged(v) => {
                trace!("Volume changed to {} for {}", v, event.id);
            }
            PlayerEventType::EndOfStream => {
                trace!("End of stream for {}", event.id);
                if let Some(v) = instances
                    .read()
                    .expect("Can't read instance!")
                    .get(&event.id)
                {
                    v.end_of_stream();
                } else {
                    debug!("Instance not found {}", event.id);
                }
            }
            PlayerEventType::StateChanged(state) => {
                trace!("State changed for {}: {:?}", event.id, state);
                if let Some(v) = instances
                    .read()
                    .expect("Can't read instance!")
                    .get(&event.id)
                {
                    v.send_playstate_change(playback_to_public_state(state));
                }
            }
            PlayerEventType::PositionUpdated(time) => {
                if let Some(time) = time.mseconds() {
                    Instance::send_position_update(event.id, time as u32);
                }
            }
            PlayerEventType::MediaInfoUpdated => (), // silence
            PlayerEventType::Error(e) => {
                let mut retry = false;
                if let Some(err) = e.kind::<ResourceError>() {
                    match err {
                        ResourceError::NotAuthorized | ResourceError::NotFound => {
                            info!(
                                "Unable to read resource:{:?} for instance {}",
                                err, event.id
                            );
                            retry = true;
                        }
                        v => warn!("Resource error {:?} for instance {}", v, event.id),
                    }
                } else {
                    let err_str = e.to_string();
                    warn!(
                        "Internal playback error for instance {}\nDetails:{} \\Details",
                        event.id, err_str
                    );
                    if e.is::<PlayerError>() {
                        if err_str.contains("Forbidden (403)") {
                            debug!("Arcane magic detected URL Forbidden error, retrying..");
                            retry = true;
                        }
                    }
                }
                if retry {
                    if let Some(v) = instances
                        .read()
                        .expect("Can't read instance!")
                        .get(&event.id)
                    {
                        if let Err(e) = v.force_song_retry() {
                            warn!("Couldn't restart playback: {}", e);
                        }
                    }
                }
            }
        }
        Ok(())
    });

    runtime.spawn(stream);
    debug!("Running ");
    Ok(())
}
