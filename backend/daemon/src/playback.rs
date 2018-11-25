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
use vlc::{self, sys, Instance, Media, MediaPlayer, MediaPlayerAudioEx};

macro_rules! lock {
    ($x:expr) => {
        lock_r!($x)?
    };
}

macro_rules! lock_r {
    ($x:expr) => {
        $x.lock().map_err(|_| PlaybackErr::LockError)
    };
}

#[derive(Fail, Debug)]
pub enum PlaybackErr {
    #[fail(display = "Player Instance error {}", _0)]
    Instance(&'static str),
    #[fail(display = "Playback Media error {}", _0)]
    Media(&'static str),
    #[fail(display = "Player error {}", _0)]
    Player(&'static str),
    #[fail(display = "Error on aquiring mutex for media")]
    LockError,
}

pub struct Player {
    media: Mutex<Option<Media>>,
    instance: Arc<Mutex<Instance>>,
    player: Mutex<MediaPlayer>,
}

impl Player {
    /// Creates a new instance of libvlc
    pub fn create_instance() -> Fallible<Arc<Mutex<Instance>>> {
        let mut args: Vec<String> = Vec::new();
        args.push("--no-video".to_string());
        let instance = Instance::with_args(Some(args))
            .ok_or(PlaybackErr::Instance("can't create a new player instance"))?;
        Ok(Arc::new(Mutex::new(instance)))
    }

    /// Create new Player with given instance
    pub fn new(instance: Arc<Mutex<Instance>>) -> Fallible<Player> {
        debug!("player init");
        let instance_c = instance.clone();
        let instance_ = lock!(instance_c);
        Ok(Player {
            media: Mutex::new(None),
            player: Mutex::new(
                MediaPlayer::new(&*instance_).ok_or(PlaybackErr::Player("can't create player"))?,
            ),
            instance,
        })
    }

    /// Set volume
    pub fn set_volume(&self, volume: i32) -> Fallible<()> {
        Ok(lock!(self.player)
            .set_volume(volume)
            .map_err(|_| PlaybackErr::Player("can't set volume"))?)
    }

    /// Set url as media
    pub fn set_url(&mut self, url: &str) -> Fallible<()> {
        let mut lock = lock!(self.media);
        *lock = Some(
            Media::new_location(&*lock!(self.instance), url)
                .ok_or(PlaybackErr::Media("can't create media for url"))?,
        );
        lock!(self.player).set_media(lock.as_ref().unwrap());

        Ok(())
    }

    /// Set file to play
    pub fn set_file(&mut self, file: &Path) -> Fallible<()> {
        let mut lock = lock!(self.media);
        *lock = Some(
            Media::new_path(&*lock!(self.instance), file)
                .ok_or(PlaybackErr::Media("can't create media for file"))?,
        );
        let media_ref = lock.as_ref().unwrap();
        media_ref.parse_async();
        lock!(self.player).set_media(media_ref);

        Ok(())
    }
    /// Play current media
    pub fn play(&self) -> Fallible<()> {
        match lock!(self.player).play() {
            Ok(_) => Ok(()),
            Err(_) => Err(PlaybackErr::Player("can't play media").into()),
        }
    }
    /// Check whether player is playing
    pub fn is_playing(&self) -> Fallible<bool> {
        Ok(lock!(self.player).is_playing())
    }

    /// Get position of player from 0.0 to 1.0 in media
    pub fn get_position(&self) -> Fallible<f32> {
        Ok(match lock!(self.player).get_position() {
            Some(v) => v,
            None => 0.0,
        })
    }
    /// Check whether current media has ended playing, false when no media is set
    pub fn ended(&self) -> Fallible<bool> {
        Ok(match *lock!(self.media) {
            Some(ref m) => m.state() == vlc::State::Ended,
            None => false,
        })
    }

    pub fn get_audio_modules(&self) -> Fallible<Vec<AudioOutput>> {
        let mut modules = Vec::new();
        unsafe {
            let p0 = sys::libvlc_audio_output_list_get(lock!(self.instance).raw());

            let mut pnext = p0;

            while !pnext.is_null() {
                modules.push(AudioOutput {
                    name: CStr::from_ptr((*pnext).psz_name)
                        .to_string_lossy()
                        .into_owned(),
                    description: CStr::from_ptr((*pnext).psz_description)
                        .to_string_lossy()
                        .into_owned(),
                });
                pnext = (*pnext).p_next;
            }
            sys::libvlc_audio_output_list_release(p0);
        }
        Ok(modules)
    }

    pub fn set_audio_module(&self, audio_output: &AudioOutput) {
        unsafe {
            sys::libvlc_audio_output_set(
                lock_r!(self.player).unwrap().raw(),
                CString::new(audio_output.name.clone()).unwrap().into_raw(),
            );
        }
    }

    pub fn get_audio_device_list(&self, module: &AudioOutput) -> Fallible<Vec<AudioDevice>> {
        let mut devices = Vec::new();
        unsafe {
            let p0 = sys::libvlc_audio_output_device_list_get(
                lock!(self.instance).raw(),
                CString::new(module.name.clone()).unwrap().into_raw(),
            );

            let mut pnext = p0;

            while !pnext.is_null() {
                devices.push(AudioDevice {
                    device: CStr::from_ptr((*pnext).psz_device)
                        .to_string_lossy()
                        .into_owned(),
                    description: CStr::from_ptr((*pnext).psz_description)
                        .to_string_lossy()
                        .into_owned(),
                });
                pnext = (*pnext).p_next;
            }
            sys::libvlc_audio_output_device_list_release(p0);
        }
        Ok(devices)
    }

    pub fn get_audio_device_list_enum(&self) -> Fallible<Vec<AudioDevice>> {
        let mut devices = Vec::new();
        unsafe {
            let p0 = sys::libvlc_audio_output_device_enum(lock!(self.player).raw());

            let mut pnext = p0;

            while !pnext.is_null() {
                devices.push(AudioDevice {
                    device: CStr::from_ptr((*pnext).psz_device)
                        .to_string_lossy()
                        .into_owned(),
                    description: CStr::from_ptr((*pnext).psz_description)
                        .to_string_lossy()
                        .into_owned(),
                });
                pnext = (*pnext).p_next;
            }
            sys::libvlc_audio_output_device_list_release(p0);
        }
        Ok(devices)
    }
}

/// Audio output module
#[derive(Debug)]
pub struct AudioOutput {
    pub name: String,
    pub description: String,
}

/// Audio device
#[derive(Debug)]
pub struct AudioDevice {
    pub device: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn libvlc_test_audio_modules() {
        let instance = Player::create_instance().unwrap();
        let mut player = Player::new(instance).unwrap();

        let modules = player.get_audio_modules().unwrap();
        println!("Modules: {:?}", modules);

        let module_pulse = modules.iter().find(|v| v.name == "pulse").unwrap();

        player
            .set_url("https://cdn.online-convert.com/example-file/audio/ogg/example.ogg")
            .unwrap();

        player.set_audio_module(module_pulse);

        let devices = player.get_audio_device_list_enum().unwrap();
        println!("Devices:");
        for device in devices {
            println!("{:?}", device);
        }
        player.play().unwrap();
        thread::sleep(Duration::from_secs(1));

        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    fn libvlc_minimal_playback() {
        // Create an instance
        let instance = Instance::with_args(None).unwrap();
        // Create a media from a file
        //https://cdn.online-convert.com/example-file/audio/ogg/example.ogg
        let md = Media::new_path(&instance, "example.ogg").unwrap();
        println!("State: {:?}", md.state());
        md.parse();
        while !md.is_parsed() {
            thread::sleep(Duration::from_millis(10));
        }
        println!("State: {:?}", md.state());
        println!("Parsed: {}", md.is_parsed());
        println!("Tracks: {:?}", md.tracks());
        println!("Meta: {:?}", md.get_meta(vlc::Meta::Title));
        if let Some(duration) = md.duration() {
            println!("Duration: {}ms", duration);
        } else {
            println!("No duration!");
        }
        assert_eq!(Some(34000), md.duration());
        // Create a media player
        let mdp = MediaPlayer::new(&instance).unwrap();
        mdp.set_media(&md);

        assert_eq!(Some(-1), mdp.title_count(), "movie title count");
        assert_eq!(0, mdp.has_vout());
        assert_eq!(None, mdp.get_position());

        // Start playing
        mdp.play().unwrap();
        assert_eq!(true, mdp.will_play(), "will play");
        println!("State: {:?}", md.state());
        let mut was_playing = false;
        // Wait for 10 seconds
        while md.state() != vlc::State::Ended {
            if md.state() == vlc::State::Playing {
                was_playing = true;
                assert!(mdp.get_position().is_some(), "has position");
            }
            println!("State: {:?}", md.state());
            /*if let Some(duration) = md.duration() {
                println!("Duration: {}ms",duration);
            }*/
            if let Some(position) = mdp.get_position() {
                println!("Position: {}", position);
            }
            //println!("Tracks: {:?}",md.tracks());
            thread::sleep(Duration::from_millis(500));
        }
        assert!(mdp.get_position().is_some(), "has position");
        assert!(was_playing, "was playing");
    }

    #[test]
    fn libvlc_version() {
        println!("Version : {}", vlc::version());
        println!("Compiler : {}", vlc::compiler());
    }
}
