//! Main of all commands that **require** the SUID flag.

#![feature(never_type)]
#![feature(os_str_display)]

mod login;
mod su;

use std::env;
use utils::error;

fn main() {
    let mut args = env::args_os();
    let bin = args
        .next()
        .and_then(|s| s.into_string().ok())
        .unwrap_or_else(|| {
            error("mutils", "missing binary name");
        });
    match bin.as_str() {
        "login" => login::main(args),
        "su" => su::main(args),
        _ => error("mutils", "invalid binary name"),
    }
}
