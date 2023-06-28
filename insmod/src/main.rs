//! The `insmod` command loads a module from a file.

use std::env;
use std::ffi::c_long;
use std::fs::File;
use std::io::Error;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::process::exit;
use std::ptr::null;
use utils::syscall;

/// The ID of the `finit_module` system call.
const FINIT_MODULE_ID: c_long = 0x15e;

/// Prints usage.
fn print_usage() {
    println!("Usage:");
    println!(" insmod <filename> [params]");
    println!();
    println!("Loads a kernel module from the given file");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        exit(1);
    }

    let filepath = PathBuf::from(&args[1]);
    let file = File::open(&filepath).unwrap_or_else(|e| {
        eprintln!("insmod: cannot open file `{}`: {}", filepath.display(), e);
        exit(1);
    });

    // TODO handle parameters
    let ret = unsafe { syscall(FINIT_MODULE_ID, file.as_raw_fd(), null::<u8>(), 0) };
    if ret < 0 {
        eprintln!(
            "insmod: cannot load module `{}`: {}",
            filepath.display(),
            Error::last_os_error()
        );
        exit(1);
    }
}
