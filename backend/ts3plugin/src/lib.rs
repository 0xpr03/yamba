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
    // Return: allowed, message
    pub fn heartbeat(&mut self, id : String) -> RpcRequest<(bool)>;
    // Return: allowed, message, Volume [0 - 100]
    pub fn volume_get(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, i32)>;
    // Return: allowed, message, success
    pub fn volume_set(&mut self, id : String, invokerName : String, invokerGroups : String, volume : i32) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn volume_lock(&mut self, id : String, invokerName : String, invokerGroups : String, lock : bool) -> RpcRequest<(bool, String, bool)>;

    // Return: allowed, message, title
    pub fn track_get(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, String)>;
    // Return: allowed, message, success
    pub fn track_next(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_previous(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_resume(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_pause(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn track_stop(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;

    // Return: allowed, message, name
    pub fn playlist_get(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, String)>;
    // Return: allowed, message, success
    pub fn playlist_clear(&mut self, id : String, invokerName : String, invokerGroups : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_lock(&mut self, id : String, invokerName : String, invokerGroups : String, lock : bool) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_queue(&mut self, id : String, invokerName : String, invokerGroups : String, url : String) -> RpcRequest<(bool, String, bool)>;
    // Return: allowed, message, success
    pub fn playlist_load(&mut self, id : String, invokerName : String, invokerGroups : String, playlist_name : String) -> RpcRequest<(bool, String, bool)>;
});

lazy_static! {
    static ref PORT: u16 = env::var("CALLBACK_YAMBA")
        .unwrap_or("1337".to_string())
        .parse::<u16>()
        .unwrap_or(1337);
    pub static ref ID: Option<i32> = env::var("ID_YAMBA")
        .unwrap_or("1337".to_string())
        .parse::<i32>()
        .map(|v| Some(v))
        .unwrap_or(None);
}

#[derive(Debug)]
struct MyTsPlugin {
    killer: Sender<()>,
    rpc_host: String,
}

const PLUGIN_NAME_I: &'static str = env!("CARGO_PKG_NAME");

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
        api.log_or_print("Initializing ", PLUGIN_NAME_I, LogLevel::Debug);

        let rpc_host: String;
        rpc_host = format!("http://localhost:{}/", PORT.to_string());

        let transport = HttpTransport::new().standalone().unwrap();
        let transport_handle = transport.handle(&rpc_host).unwrap();
        let client = BackendRPCClient::new(transport_handle);
        let client_mut = Mutex::from(client);

        let (sender, receiver) = channel();
        let id_copy = ID.clone();
        thread::spawn(move || {
            let mut failed_heartbeats = 0;
            while receiver.recv_timeout(Duration::from_secs(1)).is_err() {
                if let Ok(mut client_lock) = client_mut.lock() {
                    match client_lock.heartbeat(id_copy).call() {
                        Ok(res) => {
                            failed_heartbeats = 0;
                            TsApi::static_log_or_print(
                                format!("Server responded with {}", res),
                                PLUGIN_NAME_I,
                                LogLevel::Debug,
                            );
                        }
                        Err(_) => {
                            failed_heartbeats += 1;
                            TsApi::static_log_or_print(
                                format!(
                                    "Backend server did not respond {} times!",
                                    failed_heartbeats
                                ),
                                PLUGIN_NAME_I,
                                LogLevel::Debug,
                            );
                        }
                    }
                }
            }
        });

        let me = MyTsPlugin {
            killer: sender,
            rpc_host: rpc_host,
        };

        api.log_or_print(format!("{:?}", me), PLUGIN_NAME_I, LogLevel::Debug);

        Ok(Box::new(me))
    }

    fn shutdown(&mut self, api: &mut TsApi) {
        self.killer.send(()).unwrap();
        api.log_or_print("Shutdown", PLUGIN_NAME_I, LogLevel::Info);
    }

    fn message(
        &mut self,
        api: &mut ::TsApi,
        server_id: ::ServerId,
        invoker: ::Invoker,
        target: ::MessageReceiver,
        message: String,
        ignored: bool,
    ) -> bool {
        if let Some(server) = api.get_server(server_id) {
            if Ok(invoker.get_id()) == server.get_own_connection_id() {
                return false;
            }

            if let Some(connection) = server.get_connection(invoker.get_id()) {
                let r_vol_set =
                    RegexSet::new(&[r"^!v (\d)", r"^!vol (\d)", r"^!volume (\d)"]).unwrap();
                let r_vol_get = RegexSet::new(&[r"^!v", r"^!vol", r"^!volume"]).unwrap();

                if r_vol_set.is_match(&message) {

                } else if r_vol_get.is_match(&message) {

                } else {
                    let _ = connection
                        .send_message("Sorry, I didn't get  that... Have you tried !help yet?");
                }

                if let Ok(value) = api.get_string_client_properties(
                    ClientProperties::Servergroups,
                    &invoker.get_id(),
                    &server_id,
                ) {
                    let groups = value.to_owned_string_lossy();
                    api.log_or_print(
                        format!("groups: {}", &groups),
                        PLUGIN_NAME_I,
                        LogLevel::Debug,
                    );
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
