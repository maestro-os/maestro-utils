//! TODO doc

use libc::ioctl;
use std::fs::File;
use std::io::Error;
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;

/// The `ioctl` command to read a partitions table.
const BLKRRPART: u64 = 0x125f;

// TODO

/// Makes the kernel read the partition table for the device at the given path.
pub fn read_partitions(path: &Path) -> io::Result<()> {
	let file = File::open(path)?;

	let ret = unsafe {
		ioctl(file.as_raw_fd(), BLKRRPART, 0)
	};
	if ret < 0 {
		return Err(Error::last_os_error());
	}

	Ok(())
}
