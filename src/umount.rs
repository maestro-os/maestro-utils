//! The `mount` command allows to unmount a filesystem.

use std::env::ArgsOs;
use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::io::Error;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::exit;
use utils::error;

/// Prints the command's usage.
///
/// `bin` is the name of the current binary.
fn print_usage() {
    eprintln!("Usage:");
    eprintln!(" umount [-R] dir");
    eprintln!();
    eprintln!("Options:");
    eprintln!(" -R:\tunmounts filesystems recursively");
    eprintln!(" dir:\tthe mountpoint path");
}

/// Unmounts the filesystem at the given path `target`.
pub fn unmount_fs(target: &CStr) -> io::Result<()> {
    let ret = unsafe { libc::umount(target.as_ptr() as _) };
    if ret < 0 {
        return Err(Error::last_os_error());
    }
    Ok(())
}

pub fn main(args: ArgsOs) {
    let args: Vec<_> = args.collect();
    match &args[..] {
        [opt, path] if opt == "-R" => {
            // List active mount points
            let content = fs::read_to_string("/etc/mtab")
                .unwrap_or_else(|e| error("umount", format_args!("cannot list mount points: {e}")));
            let mut mps: Vec<_> = content
                .split('\n')
                .filter_map(|entry| Some(entry.split(' ').nth(1)?.into()))
                // Filter matching paths
                .filter(|mp: &PathBuf| mp.starts_with(path))
                .collect();
            // Sort to unmount in the right order
            mps.sort_unstable();
            for mp in mps.into_iter().rev() {
                let s = CString::new(mp.as_os_str().as_bytes()).unwrap();
                unmount_fs(&s).unwrap_or_else(|e| {
                    error(
                        "umount",
                        format_args!("cannot unmount `{}`: {e}", path.display()),
                    );
                });
            }
        }
        [path] => {
            let s = CString::new(args[1].as_bytes()).unwrap();
            unmount_fs(&s).unwrap_or_else(|e| {
                error(
                    "umount",
                    format_args!("cannot unmount `{}`: {e}", path.display()),
                );
            });
        }
        _ => {
            print_usage();
            exit(1);
        }
    }
}
