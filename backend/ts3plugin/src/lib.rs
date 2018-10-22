#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;

use ts3plugin::*;

struct MyTsPlugin;

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");

impl Plugin for MyTsPlugin {
    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        api.log_or_print("Inited", PLUGIN_NAME_I, LogLevel::Info);
        Ok(Box::new(MyTsPlugin))
        // Or return Err(InitError::Failure) on failure
    }

    // Implement callbacks here

    fn shutdown(&mut self, api: &mut TsApi) {
        api.log_or_print("Shutdown", PLUGIN_NAME_I, LogLevel::Info);
    }
}

create_plugin!(
    PLUGIN_NAME_I, env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_AUTHORS"), "yamba ts3 controller",
    ConfigureOffer::No, false, MyTsPlugin);