//! The command `nologin` simply refuses login.

use std::io::Write;
use std::process::exit;
use std::{fs, io};

fn main() {
    let result = fs::read("/etc/nologin.txt");
    let msg = result
        .ok()
        .as_deref()
        .unwrap_or(b"This account is currently not available.");
    let _ = io::stdout().write_all(msg);
    exit(1);
}
