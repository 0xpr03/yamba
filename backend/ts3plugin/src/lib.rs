#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use std::sync::Mutex;
use std::{thread, time};
use ts3plugin::TsApi;
use ts3plugin::*;
use jsonrpc_client_http::HttpTransport;

jsonrpc_client!(pub struct BackendRPCClient {
    pub fn heartbeat(&mut self) -> RpcRequest<bool>;
});

#[derive(Debug)]
struct MyTsPlugin {
    app_path: String,
    conf_path: String,
    plugin_path: String,
    ressources_path: String,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");

impl Plugin for MyTsPlugin {
    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport.handle("http://localhost:1337/").unwrap();
        let client = BackendRPCClient::new(transport_handle);
        let client_mut = Mutex::from(client);

        api.log_or_print("Initializing ", PLUGIN_NAME_I, LogLevel::Debug);

        thread::spawn(move || {
            let mut failed_heartbeats = 0;
            loop {
                if let Ok(mut client_lock) = client_mut.lock() {
                    match client_lock.heartbeat().call() {
                        Ok(res) => {
                            failed_heartbeats = 0;
                            TsApi::static_log_or_print(format!("Server responded with {}", res), PLUGIN_NAME_I, LogLevel::Debug);
                        }
                        Err(_) => {
                            failed_heartbeats += 1;
                            TsApi::static_log_or_print(format!("Backend server did not respond {} times!", failed_heartbeats), PLUGIN_NAME_I, LogLevel::Debug);
                        }
                    }
                }
                thread::sleep(time::Duration::from_secs(1));
            }
        });

        let me = MyTsPlugin {
            app_path: api.get_app_path(),
            conf_path: api.get_config_path(),
            plugin_path: api.get_plugin_path(),
            ressources_path: api.get_resources_path(),
        };

        api.log_or_print(format!("{:?}", me), PLUGIN_NAME_I, LogLevel::Debug);

        Ok(Box::new(me))
    }

    // Implement callbacks here

    fn shutdown(&mut self, api: &mut TsApi) {
        api.log_or_print("Shutdown", PLUGIN_NAME_I, LogLevel::Info);
    }

    fn message(&mut self, api: &mut ::TsApi, server_id: ::ServerId, invoker: ::Invoker,
               target: ::MessageReceiver, message: String, ignored: bool) -> bool {
        if let Some(server) = api.get_server(server_id) {
            let i_name = invoker.get_name();
            let i_id = invoker.get_id();
            let i_uid = invoker.get_uid();

            unsafe {
                let raw = api.get_raw_api();
                // Gather server group info
                api.log_or_print(format!("{:?} {:?} {:?} \"{}\" ignored: {}", i_name, i_id, i_uid, message, ignored), PLUGIN_NAME_I, LogLevel::Debug);
            }
        }

        return false;
    }
}

create_plugin!(PLUGIN_NAME_I, env ! ("CARGO_PKG_VERSION"), env ! ("CARGO_PKG_AUTHORS"), "yamba ts3 controller", ConfigureOffer::No, true, MyTsPlugin);