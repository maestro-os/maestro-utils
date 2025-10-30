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

//! The `insmod` command loads a module from a file.

use std::env::ArgsOs;
use std::ffi::c_long;
use std::fs::File;
use std::io::Error;
use std::os::fd::AsRawFd;
use std::process::exit;
use std::ptr::null;
use utils::error;

/// The ID of the `finit_module` system call.
const FINIT_MODULE_ID: c_long = 0x15e;

/// Prints usage.
fn print_usage() {
    println!("Usage:");
    println!(" insmod <filename> [params]");
    println!();
    println!("Loads a kernel module from the given file");
}

pub fn main(args: ArgsOs) {
    let args: Vec<_> = args.collect();
    let [path] = args.as_slice() else {
        print_usage();
        exit(1);
    };
    let file = File::open(path).unwrap_or_else(|e| {
        error(
            "insmod",
            format_args!("cannot open file `{}`: {e}", path.display()),
        );
    });
    // TODO handle parameters
    let ret = unsafe { libc::syscall(FINIT_MODULE_ID, file.as_raw_fd(), null::<u8>(), 0) };
    if ret < 0 {
        error(
            "insmod",
            format_args!(
                "insmod: cannot load module `{}`: {}",
                path.display(),
                Error::last_os_error()
            ),
        );
    }
}
