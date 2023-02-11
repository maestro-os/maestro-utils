//! The `mount` command allows to unmount a filesystem.

use std::env;
use std::ffi::CString;
use std::ffi::c_int;
use std::io::Error;
use std::io;
use std::process::exit;

/// Prints the command's usage.
///
/// `bin` is the name of the current binary.
fn print_usage(bin: &str) {
	eprintln!("Usage:");
	eprintln!(" {} [-R] dir", bin);
	eprintln!();
	eprintln!("Options:");
	eprintln!(" -R:\tunmounts filesystems recursively");
	eprintln!(" dir:\tthe directory on which the filesystem is mounted");
}

extern "C" {
	fn umount(target: *const i8) -> c_int;
}

/// Unmounts the filesystem at the given path `target`.
pub fn unmount_fs(target: &str) -> io::Result<()> {
	let target_c = CString::new(target).unwrap();

	let ret = unsafe {
		umount(target_c.as_ptr())
	};
	if ret < 0 {
		return Err(Error::last_os_error());
	}

	Ok(())
}

fn main() {
	let args: Vec<String> = env::args().collect();

	match args.len() {
		0 => {
			print_usage("umount");
			exit(1);
		}

		2 if args[1] != "-R" => {
			unmount_fs(&args[1])
				.unwrap_or_else(|e| {
					eprintln!("{}: cannot unmount `{}`: {}", args[0], args[1], e);
					exit(1);
				});
		}

		3 if args[1] == "-R" => {
			// TODO
			todo!();
		}

		_ => {
			print_usage(&args[0]);
			exit(1);
		}
	}
}
