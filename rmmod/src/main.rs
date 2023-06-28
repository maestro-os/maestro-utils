//! The `rmmod` command unloads a module.

use std::env;
use std::ffi::c_long;
use std::ffi::CString;
use std::io::Error;
use std::process::exit;
use utils::syscall;

/// The ID of the `delete_module` system call.
const DELETE_MODULE_ID: c_long = 0x81;

/// Prints usage.
fn print_usage() {
    println!("Usage:");
    println!(" rmmod <name>");
    println!();
    println!("Unloads a kernel module");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        print_usage();
        exit(1);
    }

    let name = &args[1];
    let c_name = CString::new(name.as_bytes()).unwrap(); // TODO handle error

    let ret = unsafe { syscall(DELETE_MODULE_ID, c_name.as_ptr(), 0) };
    if ret < 0 {
        eprintln!(
            "rmmod: cannot unload module `{}`: {}",
            name,
            Error::last_os_error()
        );
        exit(1);
    }
}
