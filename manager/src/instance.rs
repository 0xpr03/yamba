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

use actix::{registry::SystemService, spawn};
use chashmap::CHashMap;
use failure::Fallible;
use futures::future::{result, Either, Future};
use hashbrown::HashMap;
use owning_ref::OwningRef;
use yamba_types::models::{
    callback::{InstanceState, Playstate, PlaystateResponse},
    *,
};

use std::ops::Deref;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock, RwLockReadGuard, Weak,
};

use crate::backend::Backend;
use crate::db::{Database, DB};
use crate::frontend;
use crate::models;
use crate::playlist::{ItemReturn, Playlist};

#[derive(Fail, Debug)]
pub enum InstanceErr {
    #[fail(display = "No instance with id {} found when expected!", _0)]
    NoInstanceFound(ID),
}

//pub type Instances = Arc<RwLock<HashMap<ID, Instance>>>;
#[derive(Clone)]
pub struct Instances {
    ins: Arc<RwLock<HashMap<ID, Instance>>>,
    pos_cache: Arc<CHashMap<ID, TimeMS>>,
    db: DB,
}

impl Deref for Instances {
    type Target = RwLock<HashMap<ID, Instance>>;
    /// Access inner instances
    fn deref(&self) -> &RwLock<HashMap<ID, Instance>> {
        &self.ins
    }
}

type InstanceRef<'a> = OwningRef<RwLockReadGuard<'a, HashMap<ID, Instance>>, Instance>;

impl Instances {
    /// Get read-ref to instance
    pub fn read<'a>(&'a self, id: &ID) -> Option<InstanceRef<'a>> {
        let instances_r = self.ins.read().expect("Can't read instance!");
        OwningRef::new(instances_r)
            .try_map(|i| match i.get(id) {
                Some(v) => Ok(v),
                None => Err(()),
            })
            .ok()
    }

    /// Read instance but return error when none is found
    pub fn read_err<'a>(&'a self, id: &ID) -> Fallible<InstanceRef<'a>> {
        self.read(id)
            .ok_or(InstanceErr::NoInstanceFound(id.clone()).into())
    }

    /// Create new instance
    pub fn create_instance(&self, new: models::NewInstance, backend: Backend) -> Fallible<ID> {
        let model = self.db.create_instance(new)?;
        let mut instances_w = self.ins.write().expect("Can't lock instance!");
        let id = model.id.clone();
        instances_w.insert(id, Instance::new(backend, self.db.clone(), self, model));
        Ok(id)
    }

    /// Returns a InstanceMin representation of all instances
    pub fn get_instances_min(&self) -> Vec<models::InstanceMin> {
        let instances_r = self.ins.read().expect("Can't read instance!");
        instances_r
            .iter()
            .map(|(_, inst)| models::InstanceMin {
                name: inst.get_name().to_string(),
                id: inst.get_id(),
                running: inst.is_running(),
            })
            .collect()
    }

    /// Start created/loaded instance  
    /// Updates startup time & writes back to DB
    pub fn start_instance(
        &self,
        id: ID,
    ) -> Fallible<impl Future<Item = (), Error = reqwest::Error>> {
        let instances_c = self.clone();
        Ok(self.read_err(&id)?.start()?.and_then(move |v| {
            instances_c
                .read_err(&id)
                .unwrap()
                .update_startup_time(Some(v.startup_time));
            Ok(())
        }))
    }

    /// New Instances-Instance
    pub fn new(db: DB) -> Instances {
        Instances {
            ins: Arc::new(RwLock::new(HashMap::new())),
            pos_cache: Arc::new(CHashMap::new()),
            db,
        }
    }

    /// Get DB
    pub fn get_db(&self) -> &DB {
        &self.db
    }

    /// Load instances from DB
    pub fn load_instances(&self, backend: Backend) -> Fallible<()> {
        let mut instances_w = self.ins.write().expect("Can't lock instance!");
        for instance in self.db.get_instances(false)? {
            let id = instance.id.clone();
            let start = instance.autostart;
            instances_w.insert(
                id,
                Instance::new(backend.clone(), self.db.clone(), self, instance),
            );
            if start {
                instances_w.get_mut(&id).unwrap().start_ignore()?;
            }
        }
        Ok(())
    }

    /// Returns playback position
    #[allow(unused)]
    pub fn get_pos(&self, id: &ID) -> Option<TimeMS> {
        self.pos_cache.get(id).map(|v| v.clone())
    }
    /// Set (new) position for instance
    pub fn set_pos(&self, id: ID, pos: TimeMS) {
        self.pos_cache.insert(id, pos);
    }
}

pub type SPlaylist = Playlist<Song>;

pub struct Instance {
    id: ID,
    name: String,
    playlist: SPlaylist,
    volume: RwLock<Volume>,
    state: AtomicUsize,
    playstate: AtomicUsize,
    backend: Backend,
    model: InstanceLoadReq,
    position: Weak<CHashMap<ID, TimeMS>>,
    db: DB,
    start_time: RwLock<Option<TimeStarted>>,
}

impl Drop for Instance {
    fn drop(&mut self) {
        match self.stop() {
            Ok(v) => {
                Backend::spawn_ignore(v);
            }
            Err(e) => warn!("Can't auto-kill instance: {}", e),
        }
        let _ = self.db.set_instance_startup(
            &self.get_id(),
            &self.start_time.read().expect("Can't read start_time!"),
        );
    }
}

#[allow(unused)]
impl Instance {
    fn new(backend: Backend, db: DB, instances: &Instances, model: models::Instance) -> Instance {
        let id = model.id.clone();
        spawn(
            frontend::WSServer::from_registry()
                .send(frontend::InstanceCreated { id: id.clone() })
                .map_err(|e| warn!("WS-Server error: {}", e)),
        );

        let (load_req, name) = model.into_InstanceLoadReq();

        Instance {
            model: load_req,
            name: name,
            id,
            db,
            start_time: RwLock::new(None),
            playlist: SPlaylist::new(),
            volume: RwLock::new(0.05),
            state: AtomicUsize::new(InstanceState::Stopped as usize),
            backend,
            playstate: AtomicUsize::new(Playstate::Stopped as usize),
            position: Arc::downgrade(&instances.pos_cache),
        }
    }

    /// Returns currently playing title
    pub fn get_current_title(&self) -> ItemReturn<Song> {
        self.playlist.get_current()
    }

    /// Format time from seconds!
    fn format_time(length: Option<u32>) -> String {
        match length {
            Some(v) => format!("{:02}:{:02}", v / 60, v % 60),
            None => String::from("--:--"),
        }
    }

    /// Returns name of Instance
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Returns whether the instance is playing
    pub fn is_playing(&self) -> bool {
        self.playstate.load(Ordering::Relaxed) == Playstate::Playing as usize
    }

    /// Randomize playlistis_playing
    pub fn shuffle(&self) {
        self.playlist.shuffle();
    }

    /// Format track to human readable display
    fn format_track(song: &Song, position: Option<TimeMS>) -> String {
        let artist = song
            .artist
            .as_ref()
            .map_or(String::new(), |a| format!(" - {}", a));
        let pos = match position {
            Some(v) => format!("{} / ", Self::format_time(Some(v / 1000))),
            None => String::new(),
        };
        let length = Self::format_time(song.length);
        format!("{} {} {}{}", song.name, artist, pos, length)
    }

    /// Get upcoming tracks formated
    pub fn get_upcoming_tracks(&self, amount: usize) -> Vec<String> {
        let amount = if amount > 30 { 30 } else { amount };

        self.playlist
            .get_next_tracks(amount)
            .iter()
            .map(|v| Self::format_track(v, None))
            .collect()
    }

    /// Get playback position for current title in instance
    pub fn get_pos(&self) -> Option<TimeMS> {
        match self.position.upgrade() {
            Some(v) => v.get(&self.id).map(|v| v.clone()),
            None => None,
        }
    }

    /// Returns formated playback info
    pub fn get_formated_title(&self) -> Fallible<String> {
        debug!(
            "Playlist size: {} Upcoming: {}",
            self.playlist.size(),
            self.playlist.amount_upcoming()
        );
        match self.playstate.load(Ordering::Relaxed) {
            x if x == (Playstate::Playing as usize) => Ok(self
                .playlist
                .get_current()
                .map_or(String::from("No current song! This is an error."), |v| {
                    Self::format_track(&v, self.get_pos())
                })),
            _ => Ok(String::from("--:--")),
        }
    }

    /// Update startup time
    pub fn update_startup_time(&self, time: Option<TimeStarted>) {
        let mut val = self.start_time.write().expect("Can't lock startup_time!");
        *val = time;
    }

    /// Start instance, ignore outcome spawn on runtime
    pub fn start_with_rt(&mut self) -> Fallible<()> {
        spawn(self.start()?.map_err(|e| error!("{:?}", e)).map(|_| ()));
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::Relaxed) != InstanceState::Stopped as usize
    }

    /// Return start future, has to be called by Instances to store state changes
    #[must_use = "Future doesn't do anything untill polled!"]
    fn start(&self) -> Fallible<impl Future<Item = InstanceLoadResponse, Error = reqwest::Error>> {
        Ok(self.backend.create_instance(&self.model)?)
    }

    /// Update volume, intendet for callbacks
    pub fn cb_update_volume(&self, volume: Volume) {
        let mut vol_w = self.volume.write().expect("Can't lock volume!");
        *vol_w = volume;
        spawn(
            frontend::WSServer::from_registry()
                .send(VolumeSetReq {
                    id: self.get_id(),
                    volume: volume.clone(),
                })
                .map_err(|e| warn!("WS-Server error: {}", e)),
        );
    }

    /// Return volume set future
    #[must_use = "Future doesn't do anything untill polled!"]
    pub fn set_volume(
        &self,
        v: Volume,
    ) -> Fallible<impl Future<Item = DefaultResponse, Error = reqwest::Error>> {
        Ok(self.backend.set_volume(&VolumeSetReq {
            id: self.get_id(),
            volume: v,
        })?)
    }

    /// Return current volume
    pub fn get_volume(&self) -> Fallible<Volume> {
        let mut vol_r = self.volume.read().expect("Can't lock volume!");
        Ok(vol_r.clone())
    }

    /// Return stop future
    #[must_use = "Future doesn't do anything untill polled!"]
    pub fn stop(&self) -> Fallible<impl Future<Item = DefaultResponse, Error = reqwest::Error>> {
        self.update_startup_time(None);
        Ok(self
            .backend
            .stop_instance(&InstanceStopReq { id: self.get_id() })?)
    }

    /// Add songs to end of queue
    pub fn add_to_queue(&self, songs: Vec<Song>) {
        self.playlist.push(songs);
    }

    /// Returns queue future.
    /// Resolves URL by cache or calling daemon.
    #[must_use = "Future doesn't do anything untill polled!"]
    pub fn queue(&self, url: String) -> Fallible<impl Future<Item = (), Error = reqwest::Error>> {
        if let Some(pl) = self.db.get_playlist_by_url(&url)? {
            trace!("Found playlist cache hit for url.");
            self.add_to_queue(pl.data);
            return Ok(Either::A(result(Ok(()))));
        }
        if let Some(song) = self.db.get_song_by_url(&url)? {
            trace!("Found song cache hit for url.");
            self.add_to_queue(vec![song]);
            return Ok(Either::A(result(Ok(()))));
        }

        let fut = self.backend.resolve_url(&ResolveRequest {
            instance: self.get_id(),
            url,
        })?;

        let tickets = self.backend.get_tickets().clone();
        let id = self.get_id();
        let fut = fut.map(move |v| {
            tickets.add_queue(id, v.ticket.clone());
            ()
        });

        Ok(Either::B(fut))
    }

    /// Start instance, ignore outcome
    pub fn start_ignore(&mut self) -> Fallible<()> {
        trace!("Startin instance {}", self.id);
        Backend::spawn_ignore(self.start()?);
        Ok(())
    }

    /// Set state, intended for backend callbacks
    pub fn cb_set_instance_state(&self, state: InstanceState) {
        self.state.store(state as usize, Ordering::Relaxed);
    }

    /// Set playback state, intended for backend callbacks
    pub fn cb_set_playback_state(&self, state: Playstate) {
        self.playstate
            .store(state.clone() as usize, Ordering::Relaxed);
        match state {
            Playstate::EndOfMedia => self.song_end(),
            ref v => debug!("Playback change: {:?}", v),
        }
        spawn(
            frontend::WSServer::from_registry()
                .send(PlaystateResponse {
                    id: self.get_id(),
                    state: state,
                })
                .map_err(|e| warn!("WS-Server error: {}", e)),
        );
    }

    /// Handle end of current song
    fn song_end(&self) {
        if let Err(e) = self.play_next_int() {
            warn!(
                "Unable to play next song, instance {}! {}",
                self.get_id(),
                e
            );
        }
    }

    /// Play next track if nothing is played
    pub fn check_playback(&self) {
        let state = self.state.load(Ordering::Relaxed);
        let playstate = self.playstate.load(Ordering::Relaxed);
        debug!("State: {} Playstate: {}", state, playstate);
        if state == InstanceState::Running as usize && playstate == Playstate::Stopped as usize {
            if let Err(e) = self.play_next_int() {
                warn!(
                    "Unable to resume playback on instance {}! {}",
                    self.get_id(),
                    e
                );
            }
        }
    }

    /// Play next track, supposed to add permission checks
    pub fn play_next(&self) -> Fallible<()> {
        self.play_next_int()
    }

    /// Play next track
    /// Note: Currently only queue
    fn play_next_int(&self) -> Fallible<()> {
        if let Some(v) = self.playlist.get_next(false) {
            let fut = self.backend.play_url(&PlaybackUrlReq {
                id: self.get_id(),
                song: v.clone(),
            })?;

            let id = self.get_id();

            Backend::spawn_on_default({
                fut.then(move |v| {
                    if let Err(e) = v {
                        warn!("Error on song playback start, instance {}! {}", id, e);
                    }
                    Ok(())
                })
            })?;
        }
        Ok(())
    }

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn get_playlist(&self) -> &SPlaylist {
        &self.playlist
    }
}
