//! The command `nologin` simply refuses login.

use std::fs;
use std::process::exit;

fn main() {
    match fs::read_to_string("/etc/nologin.txt") {
        Ok(msg) => print!("{}", msg),
        Err(_) => println!("This account is currently not available."),
    }

    exit(1);
}
