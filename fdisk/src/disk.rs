//! TODO doc

use libc::ioctl;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::fs;
use std::io::Error;
use std::io;
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::str;

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

/// Structure storing informations about a partition.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Partition {
	/// The start offset in sectors.
	pub start: u64,
	/// The size of the partition in sectors.
	pub size: u64,

	/// The partition type.
	pub part_type: String,

	/// The partition's UUID.
	pub uuid: Option<String>,

	/// Tells whether the partition is bootable.
	pub bootable: bool,
}

impl Partition {
	/// Serializes a partitions list into a sfdisk script.
	///
	/// Arguments:
	/// - `dev` is the path to the device file of the disk.
	/// - `parts` is the list of partitions.
	///
	/// The function returns the resulting script.
	pub fn serialize(dev: &str, parts: &[Self]) -> String {
		let mut script = String::new();

		// Writing header
		// TODO label
		// TODO label-id
		script += format!("device: {}\n", dev).as_str();
		script += "unit: sectors\n";
		script += "\n";

		// Writing partitions
		for (i, p) in parts.iter().enumerate() {
			script += &format!("{}{} : {}\n", dev, i, p);
		}

		script
	}

	/// Deserializes a partitions list from a given sfdisk script.
	///
	/// Arguments:
	/// - `data` is script.
	///
	/// The function returns the list of partitions.
	pub fn deserialize(data: &str) -> Vec<Self> {
		// Skip header
		let mut iter = data.split('\n');
		while let Some(line) = iter.next() {
			if line.trim().is_empty() {
				break;
			}
		}

		// Parse partitions
		let mut parts = vec![];
		for line in iter {
			if line.trim().is_empty() {
				continue;
			}

			let mut split = line.split(':').skip(1);
			let Some(values) = split.next() else {
				// TODO error
				todo!();
			};

			// Filling partition structure
			let mut part = Self::default();
			for v in values.split(',') {
				let mut split = v.split('=');
				let Some(name) = split.next() else {
					// TODO error
					todo!();
				};

				let name = name.trim();
				let value = split.next().map(|s| s.trim());

				match name {
					"start" => {
						let Some(val) = value else {
							// TODO error
							todo!();
						};
						let Ok(val) = val.parse() else {
							// TODO error
							todo!();
						};

						part.start = val;
					}

					"size" => {
						let Some(val) = value else {
							// TODO error
							todo!();
						};
						let Ok(val) = val.parse() else {
							// TODO error
							todo!();
						};

						part.size = val;
					}

					"type" => {
						let Some(val) = value else {
							// TODO error
							todo!();
						};

						part.part_type = val.to_string();
					}

					"uuid" => {
						let Some(val) = value else {
							// TODO error
							todo!();
						};

						part.uuid = Some(val.to_string());
					}

					"bootable" => part.bootable = true,

					_ => {
						// TODO error
						todo!();
					}
				}
			}

			parts.push(part);
		}

		parts
	}
}

impl fmt::Display for Partition {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			fmt,
			"start={}, size={}, type={}",
			self.start, self.size, self.part_type
		)?;

		if self.bootable {
			write!(fmt, ", bootable")?;
		}

		if let Some(ref uuid) = self.uuid {
			write!(fmt, ", uuid={}", uuid)?;
		}

		Ok(())
	}
}

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

	/// Lists disks present on the system.
	pub fn list() -> io::Result<Vec<Self>> {
		let mut disks = vec![];

		for dev in fs::read_dir("/dev")? {
			let dev_path = dev?.path();

			// Filter devices
			if !Self::is_valid(&dev_path) {
				continue;
			}

			// Getting the number of sectors on the disk
			let Ok(size) = File::open(&dev_path).map(|file| {
				crate::get_disk_size(&file)
			}) else {
				continue;
			};

			let output = Command::new("sfdisk")
				.args(&[OsString::from("-d").as_os_str(), dev_path.as_os_str()])
				.stdout(Stdio::piped())
				.stderr(Stdio::null())
				.output()?;
			if !output.status.success() {
				continue;
			}

			let Ok(script) = str::from_utf8(&output.stdout) else {
				continue;
			};
			let partitions = Partition::deserialize(script);

			disks.push(Self {
				dev_path,
				size,

				partitions,
			});
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

/// Returns the number of sectors on the given device.
pub fn get_disk_size<D: AsRawFd>(dev: &D) -> u64 {
	let mut size = 0;

	let ret = unsafe {
		ioctl(dev.as_raw_fd(), BLKGETSIZE64 as _, &mut size)
	};

	size / 512
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
