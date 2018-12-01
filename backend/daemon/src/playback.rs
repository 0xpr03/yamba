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
use std::ffi::{CStr, CString};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use failure::Fallible;
use futures::sync::mpsc::Sender;
use gst::prelude::*;
use gst_player::{self, Cast, Error, PlayerConfig};

/// Playback abstraction

#[derive(Clone, Debug)]
pub enum PlaybackState {
    Stopped,
    Paused,
    Playing,
}

#[derive(Clone, Debug)]
pub enum PlayerEventType {
    MediaInfoUpdated,
    PositionUpdated,
    EndOfStream,
    StateChanged(PlaybackState),
    VolumeChanged(f64),
    Error(gst_player::Error),
    Buffering,
}

#[derive(Clone, Debug)]
pub struct PlayerEvent {
    pub id: String,
    pub event_type: PlayerEventType,
}

#[derive(Fail, Debug)]
pub enum PlaybackErr {
    #[fail(display = "Player Instance error {}", _0)]
    Instance(&'static str),
    #[fail(display = "Playback Media error {}", _0)]
    Media(&'static str),
    #[fail(display = "Player error {}", _0)]
    Player(&'static str),
}

pub struct Player {
    player: gst_player::Player,
}

unsafe impl Send for Player {}

unsafe impl Sync for Player {}

impl Player {
    /// Create new Player with given instance
    pub fn new(events: Sender<PlayerEvent>, name: &str) -> Fallible<Player> {
        debug!("player init");
        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gst_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
        );

        // Get position updates every 250ms.
        let mut config = player.get_config();
        config.set_position_update_interval(250);
        config.set_name(name);
        player.set_config(config).unwrap();

        player.connect_uri_loaded(|player, uri| {
            player.play();
        });

        let events_clone = events.clone();
        player.connect_end_of_stream(move |player| {
            let mut events = events_clone.clone();
            let player_id = player.get_name();
            events
                .try_send(PlayerEvent {
                    id: player_id,
                    event_type: PlayerEventType::EndOfStream,
                }).unwrap();
        });

        let events_clone = events.clone();
        player.connect_media_info_updated(move |player, info| {
            let mut events = events_clone.clone();
            let player_id = player.get_name();
            events
                .try_send(PlayerEvent {
                    id: player_id,
                    event_type: PlayerEventType::MediaInfoUpdated,
                }).unwrap();
        });

        player.connect_position_updated(|player, _| {
            let player_id = player.get_name();
        });

        let events_clone = events.clone();
        player.connect_state_changed(move |player, state| {
            let mut events = events_clone.clone();
            let state = match state {
                gst_player::PlayerState::Playing => Some(PlaybackState::Playing),
                gst_player::PlayerState::Paused => Some(PlaybackState::Paused),
                gst_player::PlayerState::Stopped => Some(PlaybackState::Stopped),
                _ => None,
            };
            if let Some(s) = state {
                let player_id = player.get_name();
                events
                    .try_send(PlayerEvent {
                        id: player_id,
                        event_type: PlayerEventType::StateChanged(s),
                    }).unwrap();
            }
        });

        player.connect_volume_changed(|player| {
            let player_id = player.get_name();
        });

        let events_clone = events.clone();
        player.connect_error(move |player, error| {
            let mut events = events_clone.clone();
            let player_id = player.get_name();
            events
                .try_send(PlayerEvent {
                    id: player_id,
                    event_type: PlayerEventType::Error(error.clone()),
                }).unwrap();
        });

        Ok(Player { player })
    }

    /// Set volume
    pub fn set_volume(&self, volume: i32) {
        self.player.set_volume(f64::from(volume) / 100.0);
    }

    /// Set url as media
    pub fn set_uri(&self, url: &str) {
        self.player.set_uri(url);
        self.player.set_video_track_enabled(false);
    }

    pub fn get_position(&self) -> i32 {
        let clock = self.player.get_position();
        let mut position: i32 = 0;
        if let Some(s) = clock.seconds() {
            position += s as i32;
        }
        if let Some(m) = clock.minutes() {
            position += m as i32 * 60;
        }
        if let Some(h) = clock.hours() {
            position += h as i32 * 60 * 60;
        }
        position
    }

    /// Play current media
    pub fn play(&self) {
        self.player.play();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::sync::mpsc;
    use futures::Stream;
    use gst;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::{self, Runtime};

    #[test]
    fn test_playback() {
        println!("test");
        gst::init().unwrap();
        let (mut sender, recv) = mpsc::channel::<PlayerEvent>(20);

        let player = Player::new(sender.clone(), "player1").unwrap();
        let mut runtime = Runtime::new().unwrap();

        let stream = recv.for_each(move |event| {
            println!("Event: {:?}", event);
            Ok(())
        });

        runtime.spawn(stream);
        player.set_uri("https://cdn.online-convert.com/example-file/audio/ogg/example.ogg");
        player.play();
        let mut vol = 0;
        sender.try_send(PlayerEvent {
            id: String::from("test"),
            event_type: PlayerEventType::Buffering,
        });
        loop {
            if vol > 100 {
                vol = 0;
            }
            player.set_volume(vol);
            vol += 10;
            thread::sleep(Duration::from_millis(500));
            println!("Volume: {}", vol);
            sender.try_send(PlayerEvent {
                id: String::from("test"),
                event_type: PlayerEventType::Buffering,
            });
        }
    }
}
