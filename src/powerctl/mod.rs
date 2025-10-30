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

//! The `powerctl` command implements power control features such as shutdown, reboot, halt, etc...

mod power;

use power::halt;
use power::poweroff;
use power::reboot;
use power::suspend;
use std::env::ArgsOs;
use std::process::exit;
use utils::error;

/// Prints command usage.
///
/// `name` is the name of the binary.
fn print_usage(name: &str) {
    println!("Usage:");
    println!(" {name} [-f] [-n]");
    println!();
    println!("Controls the system's power.");
    println!();
    println!("Options:");
    println!(" -f\tforce operation without stopping services");
    println!(" -n\tdon't synchronize storage");
}

/// Input arguments.
struct Args {
    /// If true, the command forces the operation and doesn't stop services.
    force: bool,
    /// If true, the command doesn't sync storage.
    no_sync: bool,
}

/// Parses arguments from the given array.
fn parse_args(args: ArgsOs) -> Args {
    let mut res = Args {
        force: false,
        no_sync: false,
    };
    for arg in args {
        match arg.to_str() {
            Some("-f" | "--force") => res.force = true,
            Some("-n" | "--no-sync") => res.no_sync = true,
            _ => error(
                "powerctl",
                format_args!("invalid argument `{}`", arg.display()),
            ),
        }
    }
    res
}

pub fn main(bin: &str, args: ArgsOs) {
    // Parse arguments
    let args = parse_args(args);
    if !args.force {
        // TODO Stop services
    }
    if !args.no_sync {
        // TODO Sync storage
    }
    match bin {
        "shutdown" | "poweroff" => poweroff(),
        "reboot" => reboot(),
        "halt" => halt(),
        "suspend" => suspend(),
        _ => {
            print_usage(bin);
            exit(1);
        }
    }
}
