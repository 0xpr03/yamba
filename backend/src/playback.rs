/*
 *  This file is part of yamba.
 *
 *  Foobar is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Foobar is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Foobar.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::path::Path;

use vlc::{self,Instance, Media, MediaPlayer};
use std::thread;
use failure::Fallible;

#[derive(Fail, Debug)]
pub enum PlaybackErr {
    #[fail(display = "Player Instance error {}", _0)]
    Instance(&'static str),
    #[fail(display = "Playback Media error {}", _0)]
    Media(&'static str),
    #[fail(display = "Player error {}", _0)]
    Player(&'static str)
}

pub struct Player<'a> {
    media: Option<Media>,
    instance: &'a Instance,
    player: MediaPlayer
}

impl <'a>Player<'a> {
    pub fn create_instance() -> Fallible<Instance> {
        let instance = Instance::new().ok_or(PlaybackErr::Instance("can't create a new player instance"))?;
        Ok(instance)
    }

    pub fn new(instance: &'a Instance) -> Fallible<Player<'a>> {
        Ok(Player {
            media: None,
            player: MediaPlayer::new(instance).ok_or(PlaybackErr::Player("can't create player"))?,
            instance,
        })
    }
    
    pub fn set_file(&mut self, file: &Path) -> Fallible<()> {
        self.media = Some(Media::new_path(self.instance,file).ok_or(PlaybackErr::Media("can't create media for file"))?);
        self.player.set_media(self.media.as_ref().unwrap());
        
        Ok(())
    }
    
    pub fn play(&self) -> Fallible<()> {
        self.player.play().unwrap();
        Ok(())
    }
    
    pub fn is_playing(&self) -> bool {
        self.player.is_playing()
    }
    
    pub fn get_position(&self) -> f32 {
        match self.player.get_position() {
            Some(v) => v,
            None => 0.0
        }
    }
    
    pub fn ended(&self) -> bool {
        match self.media {
            Some(ref m) => m.state() == vlc::State::Ended,
            None => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn libvlc_minimal_playback() {
        // Create an instance
        let instance = Instance::new().unwrap();
        // Create a media from a file
        //https://cdn.online-convert.com/example-file/audio/ogg/example.ogg
        let md = Media::new_path(&instance, "example.ogg").unwrap();
        println!("State: {:?}",md.state());
        md.parse();
        while !md.is_parsed() {
            thread::sleep(Duration::from_millis(10));
        }
        println!("State: {:?}",md.state());
        println!("Parsed: {}",md.is_parsed());
        println!("Tracks: {:?}",md.tracks());
        println!("Meta: {:?}",md.get_meta(vlc::Meta::Title));
        if let Some(duration) = md.duration() {
            println!("Duration: {}ms",duration);
        } else {
            println!("No duration!");
        }
        assert_eq!(Some(34000),md.duration());
        // Create a media player
        let mdp = MediaPlayer::new(&instance).unwrap();
        mdp.set_media(&md);
        
        
        assert_eq!(Some(-1),mdp.title_count(),"movie title count");
        assert_eq!(0,mdp.has_vout());
        assert_eq!(None,mdp.get_position());

        // Start playing
        mdp.play().unwrap();
        assert_eq!(true,mdp.will_play(),"will play");
        println!("State: {:?}",md.state());
        let mut was_playing = false;
        // Wait for 10 seconds
        while md.state() != vlc::State::Ended {
            if md.state() == vlc::State::Playing {
                was_playing = true;
                assert!(mdp.get_position().is_some(),"has position");
            }
            println!("State: {:?}",md.state());
            /*if let Some(duration) = md.duration() {
                println!("Duration: {}ms",duration);
            }*/
            if let Some(position) = mdp.get_position() {
                println!("Position: {}",position);
            }
            //println!("Tracks: {:?}",md.tracks());
            thread::sleep(Duration::from_millis(500));
        }
        assert!(mdp.get_position().is_some(),"has position");
        assert!(was_playing,"was playing");
    }
    
    #[test]
    fn libvlc_version() {
        println!("Version : {}", vlc::version());
        println!("Compiler : {}", vlc::compiler());
    }
}
