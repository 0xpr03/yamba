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
use futures::sync::mpsc::Sender;
use glib::FlagsClass;
use gst;
use gst::prelude::*;
use gst_player::{self, Cast};

use std::path::Path;
use std::sync::{Arc, RwLock};

use daemon::instance::ID;

/// Playback abstraction

#[derive(PartialEq, Debug)]
pub struct Position {
    pub hours: u64,
    pub minutes: u64,
    pub seconds: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    Stopped,
    Paused,
    Playing,
}

/// Player event types
#[derive(Debug)]
pub enum PlayerEventType {
    UriLoaded,
    MediaInfoUpdated,
    PositionUpdated(gst::ClockTime),
    EndOfStream,
    StateChanged(PlaybackState),
    VolumeChanged(f64),
    Error(gst_player::Error),
}

/// Player event
#[derive(Debug)]
pub struct PlayerEvent {
    pub id: ID,
    pub event_type: PlayerEventType,
}

#[derive(Fail, Debug)]
pub enum PlaybackErr {
    #[fail(display = "Player error {}", _0)]
    Player(&'static str),
    #[fail(display = "Can't play file, non UTF8 path: {}", _0)]
    InvalidFilePath(String),
    #[fail(display = "Error during GST call: {}", _0)]
    GST(&'static str),
}

/// Player struct holding the player for one instance
pub struct Player {
    player: gst_player::Player,
    pulsesink: gst::Element,
    volume: RwLock<f64>,
    state: Arc<RwLock<PlaybackState>>,
}

unsafe impl Send for Player {}

unsafe impl Sync for Player {}

pub type PlaybackSender = Sender<PlayerEvent>;

impl Player {
    /// Create new Player with given instance
    pub fn new(events: PlaybackSender, id: ID, volume: f64) -> Fallible<Player> {
        debug!("player init");

        let dispatcher = gst_player::PlayerGMainContextSignalDispatcher::new(None);
        let player = gst_player::Player::new(
            None,
            Some(&dispatcher.upcast::<gst_player::PlayerSignalDispatcher>()),
        );

        let mut config = player.get_config();

        let name = Player::get_name_by_id(&id);

        config.set_name(&name);

        // Get position updates every 1000ms.
        config.set_position_update_interval(1000);
        player.set_config(config).unwrap();

        let playbin = player.get_pipeline();
        let flags = playbin.get_property("flags")?;
        let flags_class = FlagsClass::new(flags.type_())
            .ok_or(PlaybackErr::Player("Unable to create new flags obj!"))?;
        let flags = flags_class
            .builder_with_value(flags)
            .ok_or(PlaybackErr::Player("Couldn't create flags builder!"))?
            .unset_by_nick("text")
            .unset_by_nick("video")
            .build()
            .ok_or(PlaybackErr::Player("Couldn't build flags!"))?;

        playbin.set_property("flags", &flags)?;

        let pulsesink = gst::ElementFactory::make("pulsesink", name.as_str())
            .ok_or(PlaybackErr::GST("Couldn't create pulsesink"))?;
        playbin
            .set_property("audio-sink", &pulsesink)
            .map_err(|_| PlaybackErr::GST("Couldn't set audio sink to playbin!"))?;

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_uri_loaded(move |_, _| {
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::UriLoaded,
                })
                .unwrap();
        });

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_end_of_stream(move |_| {
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::EndOfStream,
                })
                .unwrap();
        });

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_media_info_updated(move |_, _| {
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::MediaInfoUpdated,
                })
                .unwrap();
        });

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_position_updated(move |_, time| {
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::PositionUpdated(time),
                })
                .unwrap();
        });

        let events_clone = events.clone();
        let id_clone = id.clone();

        let state_store = Arc::new(RwLock::new(PlaybackState::Stopped));
        let state_clone = state_store.clone();
        player.connect_state_changed(move |_, state| {
            let mut events = events_clone.clone();
            debug!("state changed: {:?}", state);
            let state = match state {
                gst_player::PlayerState::Playing => Some(PlaybackState::Playing),
                gst_player::PlayerState::Paused => Some(PlaybackState::Paused),
                gst_player::PlayerState::Stopped => Some(PlaybackState::Stopped),
                _ => None,
            };
            if let Some(s) = state {
                let id = id_clone.clone();
                *state_clone.write().expect("Can't read state store") = s.clone();
                events
                    .try_send(PlayerEvent {
                        id,
                        event_type: PlayerEventType::StateChanged(s),
                    })
                    .unwrap();
            }
        });

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_volume_changed(move |player| {
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::VolumeChanged(player.get_volume()),
                })
                .unwrap();
        });

        let events_clone = events.clone();
        let id_clone = id.clone();
        player.connect_error(move |_, error| {
            debug!("Error event: {:?}", error);
            let mut events = events_clone.clone();
            let id = id_clone.clone();
            events
                .try_send(PlayerEvent {
                    id,
                    event_type: PlayerEventType::Error(error.clone()),
                })
                .unwrap();
        });

        Ok(Player {
            player,
            pulsesink,
            volume: RwLock::new(volume),
            state: state_store,
        })
    }

    /// Get player name by id, used to identify on sound systems
    pub fn get_name_by_id(id: &i32) -> String {
        format!("YAMBA_Player{}", id)
    }

    /// Set volume as value between 0 and 100
    pub fn set_volume(&self, volume: f64) {
        *self.volume.write().expect("Can't write volume") = volume;
        self.player.set_volume(volume);
    }

    /// Get volume as value between 0 and 100
    pub fn get_volume(&self) -> f64 {
        *self.volume.read().expect("Can't read volume")
    }

    /// Set uri as media
    pub fn set_uri(&self, url: &str) {
        self.player.set_uri(url);
    }

    /// Set file as media
    #[allow(dead_code)]
    pub fn set_file(&self, file: &Path) -> Fallible<()> {
        self.set_uri(&format!(
            "file://{}",
            file.to_str().ok_or(PlaybackErr::InvalidFilePath(
                file.to_string_lossy().into_owned()
            ))?
        ));
        Ok(())
    }

    /// Returns the current position as raw value
    pub fn get_position(&self) -> Position {
        let clock = self.player.get_position();
        let mut total_secs = match clock.seconds() {
            Some(v) => v,
            None => 0,
        };
        let hours = total_secs / 3600;
        total_secs %= 3600;

        Position {
            hours,
            minutes: total_secs / 60,
            seconds: total_secs % 60,
        }
    }

    /// Play current media
    pub fn play(&self) {
        self.player.play();
        self.player
            .set_volume(*self.volume.read().expect("Can't read volume!"));
    }

    /// Pause playback
    pub fn pause(&self) {
        self.player.pause();
    }

    /// Whether player is currently paused
    pub fn is_paused(&self) -> bool {
        *self.state.read().expect("Can't read player state") == PlaybackState::Paused
    }

    /// Whether player is currently playing
    pub fn is_playing(&self) -> bool {
        *self.state.read().expect("Can't read player state") == PlaybackState::Playing
    }

    /// Get playback state
    pub fn get_state(&self) -> PlaybackState {
        self.state.read().expect("Can't read player state!").clone()
    }

    /// Set pulse device for player
    pub fn set_pulse_device(&self, device: &str) -> Fallible<()> {
        self.pulsesink
            .set_property("device", &device)
            .map_err(|_| PlaybackErr::GST("Can't set pulse device!").into())
    }

    /// Stop current media
    pub fn stop(&self) {
        self.player.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audio;
    use futures::sync::mpsc;
    use futures::Stream;
    use gst;
    use std::thread;
    use std::time::Duration;
    use tokio::runtime::Runtime;

    lazy_static! {
        static ref TEST_ID: i32 = -1;
    }

    #[test]
    fn test_playback() {
        println!("testing playback");
        gst::init().unwrap();
        let (mut sender, recv) = mpsc::channel::<PlayerEvent>(20);

        let (mainloop, context) = audio::init().unwrap();

        let default_sink =
            audio::NullSink::new(mainloop.clone(), context.clone(), "default_sink").unwrap();
        default_sink.set_sink_as_default().unwrap();
        default_sink.set_source_as_default().unwrap();
        let sink = audio::NullSink::new(mainloop, context, "test1").unwrap();

        let player = Player::new(sender.clone(), TEST_ID.clone(), 0.0).unwrap();
        player.set_pulse_device(sink.get_sink_name()).unwrap();
        let mut runtime = Runtime::new().unwrap();

        let stream = recv.for_each(move |event| {
            println!("Event: {:?}", event);
            Ok(())
        });

        runtime.spawn(stream);
        player.set_uri("https://cdn.online-convert.com/example-file/audio/ogg/example.ogg");
        player.play();
        let mut vol = 0;
        sender
            .try_send(PlayerEvent {
                id: TEST_ID.clone(),
                event_type: PlayerEventType::PositionUpdated(gst::ClockTime::from_mseconds(1000)),
            })
            .unwrap();
        loop {
            if vol > 100 {
                vol = 0;
            }
            player.set_volume(f64::from(vol) / 100.0);
            vol += 10;
            thread::sleep(Duration::from_millis(500));
            println!("Volume: {}", vol);
            sender
                .try_send(PlayerEvent {
                    id: TEST_ID.clone(),
                    event_type: PlayerEventType::PositionUpdated(gst::ClockTime::from_mseconds(
                        1000,
                    )),
                })
                .unwrap();
        }
    }
}
