//! Implements disk-related utility functions.

use libc::ioctl;
use std::ffi::c_long;
use std::fs::File;
use std::fs;
use std::io::Error;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

/// ioctl macro: Command.
macro_rules! ioc {
	($a:expr, $b:expr, $c:expr, $d:expr) => {
		(($a) << 30) | (($b) << 8) | ($c) | (($d) << 16)
	};
}

/// ioctl macro: Read command.
#[macro_export]
macro_rules! ior {
	($a:expr, $b:expr, $c:ty) => {
		ioc!(2, $a, $b, std::mem::size_of::<$c>() as c_long)
	};
}

/// ioctl command: Get size of disk in number of sectors.
const BLKGETSIZE64: c_long = ior!(0x12, 114, u64);

/// Returns the number of sectors on the given device.
pub fn get_disk_size(path: &Path) -> io::Result<u64> {
	let mut size = 0;

	let metadata = fs::metadata(path)?;
	let file_type = metadata.file_type();

	if file_type.is_block_device() || file_type.is_char_device() {
		let dev = File::open(path)?;

		let ret = unsafe {
			ioctl(dev.as_raw_fd(), BLKGETSIZE64 as _, &mut size)
		};
		if ret < 0 {
			return Err(Error::last_os_error());
		}

		Ok(size / 512)
	} else if file_type.is_file() {
		Ok(metadata.len() / 512)
	} else {
		Ok(0)
	}
}
