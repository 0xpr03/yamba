Backend with core for playback

## Build
Requirements:
 - [rust](https://www.rust-lang.org)  
 - [gstreamer](http://gstreamer.freedesktop.org/)
 - [python3](https://www.python.org/) to run  
 - libpulse-dev

## Testing

Simple testing: `cargo test`  
Full output: cargo test -- --nocapture

## Docker build

 - Copy `backend.env` to `backend.default.env`
 - Adjust `backend.default.env` to your needs
 - run start.sh --build
