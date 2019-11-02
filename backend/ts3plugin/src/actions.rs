use super::*;
use failure::Fallible;
use regex::*;
use reqwest::Client;
use ts3plugin::TsApi;
use ts3plugin::*;
use yamba_types::client::*;
use serde::Serialize;

use std::str::FromStr;

/// Handle command with value
pub fn handle_action_cap<T: Serialize + FromStr + Sized>(
    cpt: Captures,
    cpt_pos: usize,
    action: &'static str,
    token: &Option<String>,
    client: &Client,
    connection: &Connection,
    api: &TsApi,
) -> Fallible<()>
where
    T::Err: ::std::fmt::Debug,
    // https://github.com/rust-lang/rust/issues/52662 can't enforce FromStr<Err: Debug>
{
    let data = match cpt[cpt_pos].parse::<T>() {
        Ok(v) => v,
        Err(e) => {
            send_msg(connection, "Invalid value provided! Try !help", api);
            api.log_or_print(
                format!("Can't parse input: {:?}", e),
                PLUGIN_NAME_I,
                LogLevel::Warning,
            );
            return Ok(());
        }
    };
    send_action(
        client,
        &ParamRequest {
            id: ID.expect("No ID!"),
            token,
            data,
        },
        action,
    )?;
    Ok(())
}

/// Handle command without vlaue
pub fn handle_action(action: &'static str, token: &Option<String>, client: &Client) -> Fallible<()> {
    send_action(
        client,
        &DefaultRequest {
            id: ID.expect("No ID!"),
            token,
        },
        action,
    )?;
    Ok(())
}

/// Send command action
pub fn send_action<T: Serialize + ?Sized>(
    client: &Client,
    data: &T,
    action: &'static str,
) -> Fallible<()> {
    match client
        .post(&format!("http://{}/actions/{}", *ADDRESS, action))
        .json(data)
        .send()
    {
        Err(e) => Err(APIErr::RequestError(e).into()),
        Ok(v) => {
            if v.status() == reqwest::StatusCode::ACCEPTED {
                Ok(())
            } else {
                Err(APIErr::NoSuccess(format!("Heartbeat failed: {}", v.status())).into())
            }
        }
    }
}
