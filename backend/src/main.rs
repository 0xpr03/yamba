extern crate vlc;

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use vlc::{self,Instance, Media, MediaPlayer};
    use std::thread;
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
