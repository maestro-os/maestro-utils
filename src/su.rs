//! `su` is a command allowing to run another command with a substitute user and group ID.

use std::env::ArgsOs;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::process::exit;
use std::process::Command;
use utils::error;

use utils::prompt::prompt;

/// The command's arguments.
#[derive(Default)]
struct Args {
    /// The user which executes the command. If None, using root.
    user: Option<OsString>,
    /// The group which executes the command. If None, using root.
    group: Option<OsString>,
    /// The shell to execute. If None, using the default.
    shell: Option<OsString>,
    /// Arguments for the command to execute.
    args: Vec<OsString>,
}

/// Parses the given CLI arguments `args` and returns their representation in the `Args` structure.
fn parse_args(args: ArgsOs) -> Args {
    let mut args = args.peekable();
    let mut result = Args::default();
    // Tells whether arguments contain initial options
    let has_options = args
        .peek()
        .map(|first_arg| first_arg.as_bytes().first().cloned() == Some(b'-'))
        .unwrap_or(false);
    // Parse options if present
    if has_options {
        for arg in args.by_ref() {
            if arg == "-" {
                break;
            }
            // TODO
        }
    }
    result.user = args.next();
    result.args = args.collect();
    result
}

pub fn main(args: ArgsOs) {
    let args = parse_args(args);

    let _user = args.user.as_deref().unwrap_or(OsStr::new("root"));
    // TODO Read user's entry
    let shell = args.shell.as_deref().unwrap_or(OsStr::new("TODO"));

    let _pass = prompt("Password: ", true);
    let correct = false; // TODO Check password against user's

    if correct {
        // TODO Change user
        // TODO use `execve` instead
        // Run the shell
        let status = Command::new(shell)
            .args(args.args)
            // TODO Set env
            .status()
            .unwrap_or_else(|e| {
                error(
                    "su",
                    format_args!("Failed to run shell `{}`: {e}", shell.display()),
                );
            });

        // Exit with the shell's status
        exit(status.code().unwrap());
    } else {
        eprintln!("su: Authentication failure");
        exit(1);
    }
}
