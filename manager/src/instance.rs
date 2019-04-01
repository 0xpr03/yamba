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
use futures::future::Future;
use hashbrown::HashMap;
use owning_ref::OwningRef;
use yamba_types::models::{
    callback::{InstanceState, Playstate},
    DefaultResponse, InstanceLoadReq, InstanceStopReq, PlaybackUrlReq, ResolveRequest,
    ResolveTicketResponse, Song, Volume, VolumeSetReq, ID,
};

use std::ops::Deref;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock, RwLockReadGuard,
};

use crate::backend::Backend;
use crate::frontend;
use crate::playlist::Playlist;

//pub type Instances = Arc<RwLock<HashMap<ID, Instance>>>;
#[derive(Clone)]
pub struct Instances {
    ins: Arc<RwLock<HashMap<ID, Instance>>>,
    pos_cache: Arc<CHashMap<ID, u64>>,
}

impl Deref for Instances {
    type Target = RwLock<HashMap<ID, Instance>>;

    fn deref(&self) -> &RwLock<HashMap<ID, Instance>> {
        &self.ins
    }
}

type InstanceRef<'a> = OwningRef<RwLockReadGuard<'a, HashMap<ID, Instance>>, Instance>;

impl Instances {
    pub fn read<'a>(&'a self, id: &ID) -> Option<InstanceRef<'a>> {
        let instances_r = self.ins.read().expect("Can't read instance!");
        OwningRef::new(instances_r)
            .try_map(|i| match i.get(id) {
                Some(v) => Ok(v),
                None => Err(()),
            })
            .ok()
    }
    pub fn new() -> Instances {
        Instances {
            ins: Arc::new(RwLock::new(HashMap::new())),
            pos_cache: Arc::new(CHashMap::new()),
        }
    }
}

pub type SPlaylist = Playlist<Song>;

pub struct Instance {
    id: ID,
    playlist: SPlaylist,
    volume: RwLock<Volume>,
    state: AtomicUsize,
    playstate: AtomicUsize,
    backend: Backend,
    model: InstanceLoadReq,
}

impl Drop for Instance {
    fn drop(&mut self) {
        match self
            .backend
            .stop_instance(&InstanceStopReq { id: self.get_id() })
        {
            Ok(v) => {
                if let Err(e) = Backend::spawn_ignore(v) {
                    warn!("Error on auto-killing instance: {}", e);
                }
            }
            Err(e) => warn!("Can't auto-kill instance: {}", e),
        }
    }
}

#[allow(unused)]
impl Instance {
    pub fn new(id: ID, backend: Backend, model: InstanceLoadReq) -> Instance {
        spawn(
            frontend::WSServer::from_registry()
                .send(frontend::InstanceCreated { id: id.clone() })
                .map_err(|e| warn!("WS-Server error: {}", e)),
        );

        Instance {
            id,
            playlist: SPlaylist::new(),
            volume: RwLock::new(0.05),
            state: AtomicUsize::new(InstanceState::Stopped as usize),
            backend,
            model,
            playstate: AtomicUsize::new(Playstate::Stopped as usize),
        }
    }

    fn format_time(length: Option<u32>) -> String {
        match length {
            Some(v) => format!("{:02}:{:02}", v / 60, v % 60),
            None => String::from("--:--"),
        }
    }

    fn format_track(song: &Song) -> String {
        let artist = song
            .artist
            .as_ref()
            .map_or(String::new(), |a| format!(" - {}", a));
        let length = Self::format_time(song.length);
        format!("{} {} {}", song.name, artist, length)
    }

    /// Get upcoming tracks formated
    pub fn get_upcoming_tracks(&self, amount: usize) -> Vec<String> {
        let amount = if amount > 30 { 30 } else { amount };

        self.playlist
            .get_next_tracks(amount)
            .iter()
            .map(|v| Self::format_track(v))
            .collect()
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
                    Self::format_track(&v)
                })),
            _ => Ok(String::from("--:--")),
        }
    }

    /// Start instance, ignore outcome spawn on runtime
    pub fn start_with_rt(&mut self) -> Fallible<()> {
        spawn(self.start()?.map_err(|e| error!("{:?}", e)).map(|_| ()));
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::Relaxed) != InstanceState::Stopped as usize
    }

    /// Return start future
    #[must_use = "Future doesn't do anything untill polled!"]
    pub fn start(&self) -> Fallible<impl Future<Item = DefaultResponse, Error = reqwest::Error>> {
        Ok(self.backend.create_instance(&self.model)?)
    }

    /// Update volume, inteded for callbacks
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
        Ok(self
            .backend
            .stop_instance(&InstanceStopReq { id: self.get_id() })?)
    }

    /// Add songs to end of queue
    pub fn add_to_queue(&self, songs: Vec<Song>) {
        self.playlist.push(songs);
    }

    /// Return queue future
    #[must_use = "Future doesn't do anything untill polled!"]
    pub fn queue(
        &self,
        url: String,
    ) -> Fallible<impl Future<Item = ResolveTicketResponse, Error = reqwest::Error>> {
        let fut = self.backend.resolve_url(&ResolveRequest {
            instance: self.get_id(),
            url,
        })?;

        let tickets = self.backend.get_tickets().clone();
        let id = self.get_id();
        let fut = fut.map(move |v| {
            tickets.add_queue(id, v.ticket.clone());
            v
        });

        Ok(fut)
    }

    /// Start instance, ignore outcome
    pub fn start_ignore(&mut self) -> Fallible<()> {
        trace!("Startin instance {}", self.id);
        Backend::spawn_ignore(self.start()?)?;
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
            v => debug!("Playback change: {:?}", v),
        }
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
        if let Some(v) = self.playlist.get_next() {
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
