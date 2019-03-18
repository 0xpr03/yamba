/*
 *  YAMBA middleware
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use failure::Fallible;
use futures::future::Future;
use hashbrown::HashMap;
use tokio::runtime::Runtime;
use yamba_types::models::{
    callback::InstanceState, DefaultResponse, InstanceLoadReq, InstanceStopReq, SongMin, ID,
};

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, RwLock,
};

use crate::backend::Backend;
use crate::playlist::Playlist;

pub type Instances = Arc<RwLock<HashMap<ID, Instance>>>;

pub type SPlaylist = Playlist<SongMin>;

pub struct Instance {
    id: ID,
    playlist: SPlaylist,
    volume: RwLock<f64>,
    state: AtomicUsize,
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

pub fn create_instances() -> Instances {
    Arc::new(RwLock::new(HashMap::new()))
}

impl Instance {
    pub fn new(id: ID, backend: Backend, model: InstanceLoadReq) -> Instance {
        Instance {
            id,
            playlist: SPlaylist::new(),
            volume: RwLock::new(0.05),
            state: AtomicUsize::new(InstanceState::Stopped as usize),
            backend,
            model,
        }
    }

    /// Start instance, ignore outcome spawn on runtime
    pub fn start_with_rt(&mut self, rt: &mut Runtime) -> Fallible<()> {
        rt.spawn(self.start()?.map_err(|e| error!("{:?}", e)).map(|_| ()));
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.state.load(Ordering::Relaxed) != InstanceState::Stopped as usize
    }

    /// Return start future
    pub fn start(&self) -> Fallible<impl Future<Item = DefaultResponse, Error = reqwest::Error>> {
        Ok(self.backend.create_instance(&self.model)?)
    }

    /// Return stop future
    pub fn stop(&self) -> Fallible<impl Future<Item = DefaultResponse, Error = reqwest::Error>> {
        Ok(self
            .backend
            .stop_instance(&InstanceStopReq { id: self.get_id() })?)
    }

    /// Start instance, ignore outcome
    pub fn start_ignore(&mut self) -> Fallible<()> {
        trace!("Startin instance {}", self.id);
        Backend::spawn_ignore(self.start()?)?;
        Ok(())
    }

    /// Set state, intended for backend callbacks
    pub fn set_state(&self, state: InstanceState) {
        self.state.store(state as usize, Ordering::Relaxed);
    }

    /// Handle end of current song
    pub fn song_end(&self) {}

    pub fn get_id(&self) -> ID {
        self.id
    }

    pub fn get_playlist(&self) -> &SPlaylist {
        &self.playlist
    }
}
