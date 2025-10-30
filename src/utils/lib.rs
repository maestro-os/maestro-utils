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

//! Global utilities for all commands.

#![feature(iter_array_chunks)]

use std::env::ArgsOs;
use std::path::PathBuf;
use std::process::exit;
use std::{env, fmt};

pub mod crc32;
pub mod disk;
pub mod guid;
pub mod partition;
pub mod prompt;
pub mod user;
pub mod util;

/// Returns the command's name along with an iterator over the command's arguments.
pub fn args() -> (String, ArgsOs) {
    let mut args = env::args_os();
    let bin = args
        .next()
        .map(PathBuf::from)
        .and_then(|p| {
            p.file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| {
            error("mutils", "missing binary name");
        });
    (bin, args)
}

/// Writes an error to stderr, then exits.
pub fn error<M: fmt::Display>(bin: &str, msg: M) -> ! {
    eprintln!("{bin}: error: {msg}");
    exit(1);
}
