//! Features utils to several commands.

use std::env::ArgsOs;
use std::path::PathBuf;
use std::process::exit;
use std::{env, fmt};

pub mod disk;
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
