//! Main of all commands that **require** the SUID flag.

#![feature(never_type)]
#![feature(os_str_display)]

mod login;
mod su;

use utils::{args, error};

fn main() {
    let (bin, args) = args();
    match bin.as_str() {
        "login" => login::main(args),
        "su" => su::main(args),
        _ => error("mutils", "invalid binary name"),
    }
}
