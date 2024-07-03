//! Features utils to several commands.

use std::fmt;
use std::process::exit;

pub mod disk;
pub mod prompt;
pub mod user;
pub mod util;

/// Writes an error to stderr, then exits.
pub fn error<M: fmt::Display>(bin: &str, msg: M) -> ! {
    eprintln!("{bin}: error: {msg}");
    exit(1);
}
