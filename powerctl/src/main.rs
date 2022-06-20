//! The `powerctl` command implements power control features such as shutdown, reboot, halt, etc...

mod power;

use power::halt;
use power::reboot;
use std::env;
use std::process::exit;

/// Prints command usage.
/// `name` is the name of the binary.
fn print_usage(name: Option<&str>) {
    // TODO
    todo!();
}

/// Structure representing input arguments.
struct Args {
    /// If true, the command forces the operation and doesn't stop services.
    force: bool,
    /// If true, the command doesn't sync storage.
    no_sync: bool,
}

/// Parses arguments from the given array.
fn parse_args(args: Vec<String>) -> Option<Args> {
    let mut err = false;
    let mut result = Args {
        force: false,
        no_sync: false,
    };

    args.into_iter()
        .skip(1)
        .for_each(| a | {
            match a.as_str() {
                "-f" | "--force" => result.force = true,
                "-n" | "--no-sync" => result.no_sync = true,

                _ => {
                    eprintln!("Invalid argument `{}`", a);
                    err = true;
                },
            }
        });

    if !err {
        Some(result)
    } else {
        None
    }
}

fn main() {
	let args: Vec<String> = env::args().collect();

    if args.len() == 0 {
        print_usage(None);
        exit(1);
    }

    // Binary name
    let bin = args[0].clone();
    // Parsing arguments
    let a = match parse_args(args) {
        Some(a) => a,
        None => exit(1),
    };

    if !a.force {
        // TODO Stop services
    }
    if !a.no_sync {
        // TODO Sync storage
    }

    match bin.as_str() {
        "shutdown" | "poweroff" => {
            // TODO
            todo!();
        },

        "reboot" => reboot(),

        "halt" => halt(),

        _ => {
            print_usage(Some(&bin));
            exit(1);
        },
    }
}
