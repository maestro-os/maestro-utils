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

//! The command `nologin` simply refuses login.

use std::io::Write;
use std::process::exit;
use std::{fs, io};

pub fn main() {
    let msg = fs::read("/etc/nologin.txt").ok();
    let msg = msg
        .as_deref()
        .unwrap_or(b"This account is currently not available.");
    let _ = io::stdout().write_all(msg);
    exit(1);
}
