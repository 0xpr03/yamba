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

#[macro_use]
extern crate ts3plugin;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate regex;

use jsonrpc_client_http::HttpTransport;
use regex::*;
use std::env;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use ts3plugin::TsApi;
use ts3plugin::*;

jsonrpc_client!(
    #[derive(Debug)]
    pub struct BackendRPCClient {
    pub fn heartbeat(&mut self, id : String) -> RpcRequest<bool>;
    // pub fn setVolume(&mut self, id : String, invokerName : String, invokerGroups : String, volume :f32) -> RpcRequest<bool>;
});

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    callback_port: String,
    id: String,
    rpc_host: String,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");
const DEFAULT_CALLBACK_PORT: &str = "1337";
const DEFAULT_ID: &str = "NO-ID";

impl Plugin for MyTsPlugin {
    fn name() -> String {
        PLUGIN_NAME_I.into()
    }
    fn version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }
    fn description() -> String {
        "yamba ts3 controller".into()
    }
    fn autoload() -> bool {
        true
    }
    fn configurable() -> ConfigureOffer {
        ConfigureOffer::No
    }

    fn new(api: &mut TsApi) -> Result<Box<MyTsPlugin>, InitError> {
        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport.handle("http://localhost:1337/").unwrap();
        let client = BackendRPCClient::new(transport_handle);
        let client_mut = Mutex::from(client);

        api.log_or_print("Initializing ", PLUGIN_NAME_I, LogLevel::Debug);
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let mut failed_heartbeats = 0;
            while receiver.recv_timeout(Duration::from_secs(1)).is_err() {
                if let Ok(mut client_lock) = client_mut.lock() {
                    match client_lock.heartbeat(id).call() {
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
            }
        });

        let me = MyTsPlugin {
            killer: sender,
            callback_port: String::from(callback_port),
            id: String::from(id),
            rpc_host: rpc_host,
        };

        api.log_or_print(format!("{:?}", me), PLUGIN_NAME_I, LogLevel::Debug);

        Ok(Box::new(me))
    }

    // Implement callbacks here

    fn shutdown(&mut self, api: &mut TsApi) {
        self.killer.send(()).unwrap();
        api.log_or_print("Shutdown", PLUGIN_NAME_I, LogLevel::Info);
    }

    fn message(&mut self, api: &mut ::TsApi, server_id: ::ServerId, invoker: ::Invoker,
               target: ::MessageReceiver, message: String, ignored: bool) -> bool {
        if let Some(server) = api.get_server(server_id){
            if Ok(invoker.get_id()) == server.get_own_connection_id(){
                return false;
            }
            if let Some(connection) = server.get_connection(invoker.get_id()) {
                if let Ok(value) = api.get_string_client_properties(ClientProperties::Servergroups,&invoker.get_id(),&server_id) {
                    let groups = value.to_owned_string_lossy();
                    api.log_or_print(format!("groups: {}",&groups), PLUGIN_NAME_I, LogLevel::Debug);
                    let _ = connection.send_message(groups);
                } else {
                    let _ = connection.send_message("Internal Error: Can't get server groups.");
                }
            }
        }
        return false;
    }
}

create_plugin!(MyTsPlugin);
