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
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{atomic, Arc, Mutex};
use std::thread;
use std::time::Duration;

use pulse;
use pulse::context::{flags, Context, State};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::{properties, Proplist};

/// Audio controller. Handling audio devices

type CMainloop = Arc<Mutex<Mainloop>>;
type CContext = Arc<Mutex<Context>>;
pub type SinkID = u32;

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
        return Err(AudioErr::SinkUnloadErr.into());
    }
    Ok(())
}

/// Get application source ID
//pub fn get_application_source_id(application: u32) -> Fallible<u32> {}

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

/// Create virtual sink
pub fn create_virtual_sink(mainloop: CMainloop, context: CContext, name: &str) -> Fallible<u32> {
    let sink_id: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    let sink_id_ref = sink_id.clone();
    let params = format!("sink_properties=device.description=Yamba_Device",);
    context
        .borrow()
        .introspect()
        .load_module("module-null-sink", &params, move |v| {
            let b: u32 = v;
            trace!("Module load: {}", v);
            *sink_id_ref.lock().unwrap() = Some(b);
        });

    while sink_id.lock().unwrap().is_none() {
        match mainloop.borrow_mut().iterate(false) {
            IterateResult::Quit(_) => return Err(AudioErr::IterateQuitErr.into()),
            IterateResult::Err(e) => return Err(AudioErr::IterateError(e).into()),
            IterateResult::Success(v) => {}
        }
    }

    let mut v = sink_id.lock().unwrap();
    let v = v.take().unwrap();
    if v == ::std::u32::MAX {
        // undocumented, happens on invalid params
        return Err(AudioErr::SinkLoadErr.into());
    }

    Ok(v)
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

        let sink_1 = create_virtual_sink(mloop.clone(), context.clone(), "sink1").unwrap();
        println!("sink1 id:{}", sink_1);

        let sink_2 = create_virtual_sink(mloop.clone(), context.clone(), "sink2").unwrap();
        println!("sink2 id:{}", sink_2);

        println!("Sinks created..");
        // let mut a = String::from("");
        // let reader = ::std::io::stdin();
        // reader.read_line(&mut a);
        thread::sleep(Duration::from_millis(500));

        delete_virtual_sink(mloop.clone(), context.clone(), sink_1).unwrap();

        println!("Deleted first sink");
        thread::sleep(Duration::from_millis(500));

        delete_virtual_sink(mloop.clone(), context.clone(), sink_2).unwrap();
        println!("Deleted second sink");

        let res = delete_virtual_sink(mloop.clone(), context.clone(), sink_2);
        assert!(res.is_err());
        println!("Tested double unload");
    }
}
