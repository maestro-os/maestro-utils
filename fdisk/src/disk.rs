//! TODO doc

use crate::partition::Partition;
use libc::ioctl;
use std::fmt;
use std::fs::File;
use std::fs;
use std::io::Error;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::path::PathBuf;

/// ioctl macro: TODO doc
macro_rules! ioc {
	($a:expr, $b:expr, $c:expr, $d:expr) => {
		(($a) << 30) | (($b) << 8) | ($c) | (($d) << 16)
	};
}

/// ioctl macro: Read command.
#[macro_export]
macro_rules! ior {
	($a:expr, $b:expr, $c:ty) => {
		ioc!(2, $a, $b, std::mem::size_of::<$c>() as u64)
	};
}

/// ioctl command: Get size of disk in number of sectors.
const BLKGETSIZE64: u64 = ior!(0x12, 114, usize);
/// ioctl command: Read a partitions table.
const BLKRRPART: u64 = 0x125f;

/// Structure representing a disk, containing partitions.
pub struct Disk {
	/// The path to the disk's device file.
	dev_path: PathBuf,
	/// The size of the disk in number of sectors.
	size: u64,

	/// The disk's partitions.
	pub partitions: Vec<Partition>,
}

impl Disk {
	/// Tells whether the device file at the given path is a valid disk.
	///
	/// This function is meant to be used when listing disks.
	fn is_valid(path: &Path) -> bool {
		let Some(path_str) = path.as_os_str().to_str() else {
			return false;
		};

		if path_str.starts_with("/dev/sd") && !path_str.contains(|c: char| c.is_numeric()) {
			return true;
		}
		if path_str.starts_with("/dev/nvme0n") && !path_str.contains('p') {
			return true;
		}

		// TODO Add USB, floppy, cdrom, etc...

		false
	}

	/// Reads a disk's informations from the given device path `dev_path`.
	///
	/// If the path doesn't point to a valid device, the function returns None.
	pub fn read(dev_path: PathBuf) -> io::Result<Option<Self>> {
		// Getting the number of sectors on the disk
		let Ok(size) = get_disk_size(&dev_path) else {
			return Ok(None);
		};

		// TODO read partitions table from disk
		let partitions = Vec::new();

		Ok(Some(Self {
			dev_path,
			size,

			partitions,
		}))
	}

	/// Writes the partition table to the disk.
	pub fn write(&self) -> io::Result<()> {
		// TODO
		todo!();
	}

	/// Lists disks present on the system.
	pub fn list() -> io::Result<Vec<Self>> {
		let mut disks = vec![];

		for dev in fs::read_dir("/dev")? {
			let dev_path = dev?.path();

			// Filter devices
			if !Self::is_valid(&dev_path) {
				continue;
			}

			let Some(dev) = Self::read(dev_path)? else {
				continue;
			};

			disks.push(dev);
		}

		Ok(disks)
	}

	/// Returns the path to the device file of the disk.
	pub fn get_dev_path(&self) -> &Path {
		&self.dev_path
	}

	/// Returns the size of the disk in number of sectors.
	pub fn get_size(&self) -> u64 {
		self.size
	}
}

impl fmt::Display for Disk {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		// TODO
		write!(fmt, "Disk TODO: TODO MiB, TODO bytes, TODO sectors")?;
		write!(fmt, "Disk model: TODO")?;
		write!(fmt, "Units: sectors of TODO * TODO = TODO bytes")?;
		write!(fmt, "Sector size (logical/physical): TODO bytes / TODO bytes")?;
		write!(fmt, "I/O size (minimum/optimal): TODO bytes / TODO bytes")?;
		write!(fmt, "Disklabel type: TODO")?;
		write!(fmt, "Disk identifier: TODO")?;

		// TODO If disk has partitions:
		write!(fmt, "\nDevice\tStart\tEnd\tSectors\tSize\tType")?;
		// TODO loop:
		write!(fmt, "/dev/TODO\tTODO\tTODO\tTODO\tTODO\tTODO")?;

		Ok(())
	}
}

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

/// Makes the kernel read the partition table for the given device.
pub fn read_partitions<D: AsRawFd>(dev: &D) -> io::Result<()> {
	let ret = unsafe {
		ioctl(dev.as_raw_fd(), BLKRRPART, 0)
	};
	if ret < 0 {
		return Err(Error::last_os_error());
	}

	Ok(())
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn partitions_serialize0() {
		let parts0 = vec![];

		let script = Partition::serialize("/dev/sda", &parts0);
		let parts1 = Partition::deserialize(&script);

		assert!(parts1.is_empty());
	}

	#[test]
	fn partitions_serialize1() {
		let parts0 = vec![Partition {
			start: 0,
			size: 1,

			part_type: "foo".to_string(),

			uuid: Some("bar".to_string()),

			bootable: false,
		}];

		let script = Partition::serialize("/dev/sda", &parts0);
		let parts1 = Partition::deserialize(&script);

		for (p0, p1) in parts0.iter().zip(&parts1) {
			assert_eq!(p0, p1);
		}
	}

	// TODO More tests (especially invalid scripts)
}
