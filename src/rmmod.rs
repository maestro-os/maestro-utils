/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! The `rmmod` command unloads a module.

use std::env::ArgsOs;
use std::ffi::CString;
use std::ffi::c_long;
use std::io::Error;
use std::os::unix::ffi::OsStrExt;
use std::process::exit;
use utils::error;

/// The ID of the `delete_module` system call.
const DELETE_MODULE_ID: c_long = 0x81;

/// Prints usage.
fn print_usage() {
    println!("Usage:");
    println!(" rmmod <name>");
    println!();
    println!("Unloads a kernel module");
}

pub fn main(args: ArgsOs) {
    let args: Vec<_> = args.collect();
    let [name] = args.as_slice() else {
        print_usage();
        exit(1);
    };
    let name = CString::new(name.as_bytes()).unwrap(); // TODO handle error
    let ret = unsafe { libc::syscall(DELETE_MODULE_ID, name.as_ptr(), 0) };
    if ret < 0 {
        error(
            "rmmod: cannot unload module `{name}`: {}",
            Error::last_os_error(),
        );
    }
}
