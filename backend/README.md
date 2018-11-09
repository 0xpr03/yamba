Backend with core for playback

## Build
Requirements:
 - [rust](https://www.rust-lang.org)  
 - [libvlc](https://wiki.videolan.org/LibVLC) or libvlc-dev on debian  
 - vlc addons (addons APT package under *debian) to run with codecs  
 - libssl-dev  
 - [python3](https://www.python.org/) to run  
 - libpulse-dev

## Testing

Simple testing: `cargo test`  
Full output: cargo test -- --nocapture
