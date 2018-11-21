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

/// Audio controller. Handling audio devices
// https://docs.rs/libpulse-sys/1.4.0/libpulse_sys/context/introspect/fn.pa_context_load_module.html

/*
https://freedesktop.org/software/pulseaudio/doxygen/async.html#conn_subsec
https://docs.rs/libpulse-sys/1.4.0/libpulse_sys/context/fn.pa_context_connect.html

*/
use failure::Fallible;

use libpulse_sys::context::introspect::pa_context_load_module;

pub struct Device {}

pub fn load_module() -> Fallible<()> {
    unsafe {
        //pulse::pa_context_load_module();
    }
}
