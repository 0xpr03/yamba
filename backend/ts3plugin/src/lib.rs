#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;

use ts3plugin::*;

struct MyTsPlugin;

impl Plugin for MyTsPlugin {
    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        api.log_or_print("Inited", "MyTsPlugin", LogLevel::Info);
        Ok(Box::new(MyTsPlugin))
        // Or return Err(InitError::Failure) on failure
    }

    // Implement callbacks here

    fn shutdown(&mut self, api: &mut TsApi) {
        api.log_or_print("Shutdown", "MyTsPlugin", LogLevel::Info);
    }
}

create_plugin!(
    "My Ts Plugin", "0.1.0", "My name", "A wonderful tiny example plugin",
    ConfigureOffer::No, false, MyTsPlugin);