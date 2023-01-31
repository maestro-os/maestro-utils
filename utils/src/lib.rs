//! This module implements features common to several commands.

pub mod prompt;
pub mod user;
pub mod util;

use std::ffi::c_long;

extern "C" {
    pub fn syscall(number: c_long, ...) -> c_long;
}
