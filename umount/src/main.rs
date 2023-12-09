//! The `mount` command allows to unmount a filesystem.

use std::env;
use std::ffi::{CStr, CString};
use std::fs;
use std::io;
use std::io::Error;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::process::exit;

/// Prints the command's usage.
///
/// `bin` is the name of the current binary.
fn print_usage(bin: &str) {
    eprintln!("Usage:");
    eprintln!(" {bin} [-R] dir");
    eprintln!();
    eprintln!("Options:");
    eprintln!(" -R:\tunmounts filesystems recursively");
    eprintln!(" dir:\tthe directory on which the filesystem is mounted");
}

/// Unmounts the filesystem at the given path `target`.
pub fn unmount_fs(target: &CStr) -> io::Result<()> {
    let ret = unsafe { libc::umount(target.as_ptr() as _) };
    if ret < 0 {
        return Err(Error::last_os_error());
    }
    Ok(())
}

/// Lists active mount points.
pub fn list_mount_points() -> io::Result<Vec<PathBuf>> {
    let content = fs::read_to_string("/etc/mtab")?;
    Ok(content
        .split('\n')
        .filter_map(|entry| Some(entry.split(' ').nth(1)?.into()))
        .collect())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 => {
            print_usage("umount");
            exit(1);
        }

        2 if args[1] != "-R" => {
            let s = CString::new(args[1].as_bytes()).unwrap();
            unmount_fs(&s).unwrap_or_else(|e| {
                eprintln!("{}: cannot unmount `{}`: {e}", args[0], args[1]);
                exit(1);
            });
        }

        3 if args[1] == "-R" => {
            let mut mount_points = list_mount_points().unwrap_or_else(|e| {
                eprintln!("{}: cannot list mount points: {e}", args[0]);
                exit(1);
            });
            mount_points.sort_unstable();

            let inner_mount_points_iter = mount_points.iter().filter(|mp| mp.starts_with(&args[1]));

            for mp in inner_mount_points_iter {
                let s = CString::new(mp.as_os_str().as_bytes()).unwrap();
                unmount_fs(&s).unwrap_or_else(|e| {
                    eprintln!("{}: cannot unmount `{}`: {e}", args[0], args[1]);
                    exit(1);
                });
            }
        }

        _ => {
            print_usage(&args[0]);
            exit(1);
        }
    }
}
