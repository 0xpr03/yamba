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

use failure::Fallible;

use std::boxed::Box;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use pulse;
use pulse::callbacks::ListResult;
use pulse::context::{flags, Context, State};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::{properties, Proplist};

/// Audio controller. Handling audio devices

pub type CMainloop = Arc<Mutex<Mainloop>>;
pub type CContext = Arc<Mutex<Context>>;
pub type SinkID = u32;

/// Pulse audio module-null-sink
pub struct NullSink {
    id: SinkID,
    name: String,
    monitor: SinkID,
    sink: SinkID,
    context: CContext,
    mainloop: CMainloop,
}

/// Enum for async filtering a sink via PA calls
#[derive(PartialEq, Eq)]
enum DeviceFilterResult {
    None,
    Running,
    Error,
    Some(SinkID),
}

impl NullSink {
    /// Create new null sink
    pub fn new<T: Into<String>>(
        mainloop: CMainloop,
        context: CContext,
        name: T,
    ) -> Fallible<NullSink> {
        let sink_id: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
        let sink_id_ref = sink_id.clone();

        let name = name.into();

        let params = format!(
            r#"sink_name="{}" sink_properties=device.description="{}""#,
            name, name,
        );

        {
            let context_l = context
                .lock()
                .map_err(|_| AudioErr::LockError("PA Context"))?;

            context_l
                .introspect()
                .load_module("module-null-sink", &params, move |v| {
                    let b: u32 = v;
                    trace!("Module load: {}", v);
                    *sink_id_ref.lock().unwrap() = Some(b);
                });

            let mut mainloop_l = mainloop
                .lock()
                .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

            while sink_id.lock().unwrap().is_none() {
                match mainloop_l.iterate(false) {
                    IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
                    IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
                    IterateResult::Success(v) => {}
                }
            }
        }

        let mut id = sink_id.lock().unwrap();
        let id = id.take().unwrap();
        if id == ::std::u32::MAX {
            // undocumented, happens on invalid params
            return Err(AudioErr::SinkLoadErr.into());
        }

        // unload sink on monitor resolve failure
        let monitor = match get_module_device(&mainloop, &context, id, ChildType::Monitor) {
            Ok(v) => v,
            Err(e) => {
                delete_virtual_sink(mainloop, context, id)?;
                return Err(e);
            }
        };

        let sink = match get_module_device(&mainloop, &context, id, ChildType::Sink) {
            Ok(v) => v,
            Err(e) => {
                delete_virtual_sink(mainloop, context, id)?;
                return Err(e);
            }
        };

        Ok(NullSink {
            monitor,
            sink,
            name,
            id: id,
            mainloop,
            context,
        })
    }

    /// Set sink volume
    pub fn mute_sink(&self, mute: bool) -> Fallible<()> {
        let success: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let success_ref = success.clone();
        let context = self
            .context
            .lock()
            .map_err(|_| AudioErr::LockError("PA Context"))?;
        context.introspect().set_sink_mute_by_index(
            self.sink,
            mute,
            Some(Box::new(move |v| {
                let b = v;
                trace!("Sink mute: {}", v);
                *success_ref.lock().unwrap() = Some(b);
            })),
        );

        let mut mainloop = self
            .mainloop
            .lock()
            .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

        while success.lock().unwrap().is_none() {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
                IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
                IterateResult::Success(_) => {}
            }
        }
        let mut v = success.lock().unwrap();
        if !v.take().unwrap() {
            return Err(AudioErr::SinkUnloadErr.into());
        }
        Ok(())
    }

    /// Set source of NullSink as default
    /// Note: Uses its name
    pub fn set_source_as_default(&self) -> Fallible<()> {
        let success: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let success_ref = success.clone();
        let mut context = self
            .context
            .lock()
            .map_err(|_| AudioErr::LockError("PA Context"))?;
        context.set_default_source(&format!("{}.monitor", self.name), move |v| {
            let b = v;
            trace!("Source default: {}", v);
            *success_ref.lock().unwrap() = Some(b);
        });

        let mut mainloop = self
            .mainloop
            .lock()
            .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

        while success.lock().unwrap().is_none() {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
                IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
                IterateResult::Success(_) => {}
            }
        }
        let mut v = success.lock().unwrap();
        // if !v.take().unwrap() {
        //     return Err(AudioErr::ResultInvalid("set source as default").into());
        // }
        Ok(())
    }

    /// Set sink of NullSink as default
    /// Note: Uses its name
    pub fn set_sink_as_default(&self) -> Fallible<()> {
        let success: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
        let success_ref = success.clone();
        let mut context = self
            .context
            .lock()
            .map_err(|_| AudioErr::LockError("PA Context"))?;
        context.set_default_sink(&self.name, move |v| {
            let b = v;
            println!("Sink default: {}", v);
            *success_ref.lock().unwrap() = Some(b);
        });

        let mut mainloop = self
            .mainloop
            .lock()
            .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

        while success.lock().unwrap().is_none() {
            match mainloop.iterate(false) {
                IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
                IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
                IterateResult::Success(_) => {}
            }
        }
        let mut v = success.lock().unwrap();
        if !v.take().unwrap() {
            return Err(AudioErr::ResultInvalid("set sink as default").into());
        }
        Ok(())
    }
}

impl Drop for NullSink {
    fn drop(&mut self) {
        if let Err(e) = delete_virtual_sink(self.mainloop.clone(), self.context.clone(), self.id) {
            warn!("Unable to delete sink {}", self.id);
        }
    }
}

#[derive(Fail, Debug)]
pub enum AudioErr {
    #[fail(display = "Iteration error: {}", _0)]
    IterateError(pulse::error::PAErr),
    #[fail(display = "Iterator was quit duration operation")]
    IterateQuitErr,
    #[fail(display = "Unable to set property for proplist")]
    PropSetErr,
    #[fail(display = "Unable to create pa context")]
    ContextCreateErr,
    #[fail(display = "Context connect error: {}", _0)]
    ContextConnectErr(pulse::error::PAErr),
    #[fail(display = "Context error while connecting: {}", _0)]
    ContextConnectingErr(&'static str),
    #[fail(display = "Couldn't unload sink")]
    SinkUnloadErr,
    #[fail(display = "Couldn't load sink, received invalid ID")]
    SinkLoadErr,
    #[fail(display = "PA call wasn't successfull for {}", _0)]
    ResultInvalid(&'static str),
    #[fail(display = "Error when retrieving sinks {}", _0)]
    SinkRetrieveErr(&'static str),
    #[fail(display = "Couldn't aquire lock for {}", _0)]
    LockError(&'static str),
}

/// Child type for module child resolving
enum ChildType {
    Sink,
    Monitor,
}

/// Retrieve device ID of module, type specified by child_type
fn get_module_device(
    mainloop: &CMainloop,
    context: &CContext,
    sink: SinkID,
    child_type: ChildType,
) -> Fallible<SinkID> {
    let success: Arc<Mutex<DeviceFilterResult>> = Arc::new(Mutex::new(DeviceFilterResult::Running));
    let success_ref = success.clone();

    let context = context
        .lock()
        .map_err(|_| AudioErr::LockError("PA Context"))?;

    match child_type {
        ChildType::Monitor => {
            context
                .introspect()
                .get_source_info_list(move |res| match res {
                    ListResult::Item(source) => {
                        if source.owner_module == Some(sink) {
                            *success_ref.lock().unwrap() = DeviceFilterResult::Some(source.index);
                        }
                    }
                    ListResult::End => {
                        let mut success_l = success_ref.lock().unwrap();
                        if *success_l == DeviceFilterResult::Running {
                            *success_l = DeviceFilterResult::None;
                        }
                    }
                    ListResult::Error => {
                        warn!("Error at fetching PA sources list");
                        *success_ref.lock().unwrap() = DeviceFilterResult::Error;
                    }
                });
        }
        ChildType::Sink => {
            context
                .introspect()
                .get_sink_info_list(move |res| match res {
                    ListResult::Item(source) => {
                        if source.owner_module == Some(sink) {
                            *success_ref.lock().unwrap() = DeviceFilterResult::Some(source.index);
                        }
                    }
                    ListResult::End => {
                        let mut success_l = success_ref.lock().unwrap();
                        if *success_l == DeviceFilterResult::Running {
                            *success_l = DeviceFilterResult::None;
                        }
                    }
                    ListResult::Error => {
                        warn!("Error at fetching PA device list");
                        *success_ref.lock().unwrap() = DeviceFilterResult::Error;
                    }
                });
        }
    }

    let mut mainloop = mainloop
        .lock()
        .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

    while *success.lock().unwrap() == DeviceFilterResult::Running {
        match mainloop.iterate(false) {
            IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
            IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
            IterateResult::Success(_) => {}
        }
    }

    let value = match *success.lock().unwrap() {
        DeviceFilterResult::Some(v) => v,
        DeviceFilterResult::Running => {
            // don't panic, or we'll not cleanup, could be enhanced with auto drop before device id resolution
            return Err(AudioErr::SinkRetrieveErr("Unreachable state: Running!").into());
        }
        DeviceFilterResult::Error => return Err(AudioErr::SinkRetrieveErr("Unknown").into()),
        DeviceFilterResult::None => {
            return Err(AudioErr::SinkRetrieveErr("No matching device found!").into())
        }
    };

    Ok(value)
}

/// Set source of process to specified source_id
pub fn set_process_source(
    mainloop: CMainloop,
    context: CContext,
    process: u32,
    source_id: SinkID,
) -> Fallible<()> {
    let success: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
    let success_ref = success.clone();
    let context = context
        .lock()
        .map_err(|_| AudioErr::LockError("PA Context"))?;
    context.introspect().move_source_output_by_index(
        process,
        source_id,
        Some(Box::new(move |v| {
            let b = v;
            trace!("Process source input set: {}", v);
            *success_ref.lock().unwrap() = Some(b);
        })),
    );

    let mut mainloop = mainloop
        .lock()
        .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

    while success.lock().unwrap().is_none() {
        match mainloop.iterate(false) {
            IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
            IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
            IterateResult::Success(_) => {}
        }
    }
    let mut v = success.lock().unwrap();
    if !v.take().unwrap() {
        return Err(AudioErr::ResultInvalid("set process source").into());
    }
    Ok(())
}

/// Delete virtual sink
fn delete_virtual_sink(mainloop: CMainloop, context: CContext, sink_id: SinkID) -> Fallible<()> {
    let success: Arc<Mutex<Option<bool>>> = Arc::new(Mutex::new(None));
    let success_ref = success.clone();
    let context = context
        .lock()
        .map_err(|_| AudioErr::LockError("PA Context"))?;
    context.introspect().unload_module(sink_id, move |v| {
        let b = v;
        trace!("Module unload: {}", v);
        *success_ref.lock().unwrap() = Some(b);
    });

    let mut mainloop = mainloop
        .lock()
        .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

    while success.lock().unwrap().is_none() {
        match mainloop.iterate(false) {
            IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
            IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
            IterateResult::Success(_) => {}
        }
    }
    let mut v = success.lock().unwrap();
    if !v.take().unwrap() {
        return Err(AudioErr::SinkUnloadErr.into());
    }
    Ok(())
}

/// Init pulse audio context
pub fn init() -> Fallible<(CMainloop, CContext)> {
    debug!("Audio context init..");
    let mut proplist = Proplist::new().unwrap();
    proplist
        .sets(properties::APPLICATION_NAME, "yamba")
        .map_err(|_| AudioErr::PropSetErr)?;
    let mainloop = Arc::new(Mutex::new(Mainloop::new().unwrap()));

    let mut mainloop_l = mainloop
        .lock()
        .map_err(|_| AudioErr::LockError("PA Mainloop"))?;

    let context = Arc::new(Mutex::new(
        Context::new_with_proplist(mainloop_l.deref(), "yamba", &proplist)
            .ok_or(AudioErr::ContextCreateErr)?,
    ));

    {
        let mut context_l = context
            .lock()
            .map_err(|_| AudioErr::LockError("PA Context"))?;

        context_l
            .connect(None, flags::NOFLAGS, None)
            .map_err(|v| AudioErr::ContextConnectErr(v))?;

        loop {
            match mainloop_l.iterate(false) {
                IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
                IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
                IterateResult::Success(_) => {}
            }
            match context_l.get_state() {
                State::Ready => {
                    break;
                }
                State::Failed => return Err(AudioErr::ContextConnectingErr("State: Failed").into()),
                State::Terminated => {
                    return Err(AudioErr::ContextConnectingErr("State: Terminated").into())
                }
                _ => {}
            }
        }
    }
    debug!("Audio context ready");

    Ok((mainloop.clone(), context))
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn init_test() {
        let (_a, _b) = init().unwrap();
        println!("Audio init test ran");
        thread::sleep(Duration::from_secs(1));
        println!("Ended");
    }

    #[test]
    fn virtual_sink_test() {
        let (mloop, context) = init().unwrap();

        let sink_1 = NullSink::new(mloop.clone(), context.clone(), "sink1").unwrap();

        let sink_2 = NullSink::new(mloop.clone(), context.clone(), "sink2").unwrap();

        sink_2.mute_sink(true).unwrap();
        sink_1.set_source_as_default().unwrap();
        sink_1.set_sink_as_default().unwrap();

        println!("Sinks created..");
        // let mut a = String::from("");
        // let reader = ::std::io::stdin();
        // reader.read_line(&mut a);
        thread::sleep(Duration::from_millis(500));

        drop(sink_1);

        println!("Deleted first sink");
        drop(sink_2);
        println!("Deleted second sink");
    }
}
