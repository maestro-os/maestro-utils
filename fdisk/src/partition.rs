//! TODO

use std::cmp::max;
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use utils::prompt::prompt;

/// Enumeration of partition table types.
pub enum PartitionTableType {
	/// Master Boot Record.
	MBR,
	/// TODO
	GPT,
}

impl PartitionTableType {
	/// Prints known partition types.
	pub fn print_partition_types(&self) {
		match self {
			Self::MBR => {
				let types: BTreeMap<u8, &'static str> = BTreeMap::from([
					(0x00, "Empty"),
					(0x01, "FAT12"),
					(0x02, "XENIX root"),
					(0x03, "XENIX usr"),
					(0x04, "FAT16 <32M"),
					(0x05, "Extended"),
					(0x06, "FAT16"),
					(0x07, "HPFS/NTFS/exFAT"),
					(0x08, "AIX"),
					(0x09, "AIX bootable"),
					(0x0a, "OS/2 Boot Manager"),
					(0x0b, "W95 FAT32"),
					(0x0c, "W95 FAT32 (LBA)"),
					(0x0e, "W95 FAT16 (LBA)"),
					(0x0f, "W95 Ext'd (LBA)"),
					(0x10, "OPUS"),
					(0x11, "Hidden FAT12"),
					(0x12, "Compaq diagnostics"),
					(0x14, "Hidden FAT16 <3"),
					(0x16, "Hidden FAT16"),
					(0x17, "Hidden HPFS/NTFS"),
					(0x18, "AST SmartSleep"),
					(0x1b, "Hidden W95 FAT3"),
					(0x1c, "Hidden W95 FAT3"),
					(0x1e, "Hidden W95 FAT1"),
					(0x24, "NEC DOS"),
					(0x27, "Hidden NTFS Win"),
					(0x39, "Plan 9"),
					(0x3c, "PartitionMagic"),
					(0x40, "Venix 80286"),
					(0x41, "PPC PReP Boot"),
					(0x42, "SFS"),
					(0x4d, "QNX4.x"),
					(0x4e, "QNX4.x 2nd part"),
					(0x4f, "QNX4.x 3rd part"),
					(0x50, "OnTrack DM"),
					(0x51, "OnTrack DM6 Aux"),
					(0x52, "CP/M"),
					(0x53, "OnTrack DM6 Aux"),
					(0x54, "OnTrackDM6"),
					(0x55, "EZ-Drive"),
					(0x56, "Golden Bow"),
					(0x5c, "Priam Edisk"),
					(0x61, "SpeedStor"),
					(0x63, "GNU HURD or Sys"),
					(0x64, "Novell Netware"),
					(0x65, "Novell Netware"),
					(0x70, "DiskSecure Mult"),
					(0x75, "PC/IX"),
					(0x80, "Old Minix"),
					(0x81, "Minix / old Linux"),
					(0x82, "Linux swap / Solaris"),
					(0x83, "Linux"),
					(0x84, "OS/2 hidden"),
					(0x85, "Linux extended"),
					(0x86, "NTFS volume set"),
					(0x87, "NTFS volume set"),
					(0x88, "Linux plaintext"),
					(0x8e, "Linux LVM"),
					(0x93, "Amoeba"),
					(0x94, "Amoeba BBT"),
					(0x9f, "BSD/OS"),
					(0xa0, "IBM Thinkpad"),
					(0xa5, "FreeBSD"),
					(0xa6, "OpenBSD"),
					(0xa7, "NeXTSTEP"),
					(0xa8, "Darwin UFS"),
					(0xa9, "NetBSD"),
					(0xab, "Darwin boot"),
					(0xaf, "HFS / HFS+"),
					(0xb7, "BSDI fs"),
					(0xb8, "BSDI swap"),
					(0xbb, "Boot Wizard hidden"),
					(0xbc, "Acronis FAT32"),
					(0xbe, "Solaris boot"),
					(0xbf, "Solaris"),
					(0xc1, "DRDOS/sec"),
					(0xc4, "DRDOS/sec"),
					(0xc6, "DRDOS/sec"),
					(0xc7, "Syrinx"),
					(0xda, "Non-FS data"),
					(0xdb, "CP/M / CTOS / ."),
					(0xde, "Dell Utility"),
					(0xdf, "BootIt"),
					(0xe0, "ST AVFS"),
					(0xe1, "DOS access"),
					(0xe3, "DOS R/O"),
					(0xe4, "SpeedStor"),
					(0xea, "Linux extended"),
					(0xeb, "BeOS fs"),
					(0xee, "GPT"),
					(0xef, "EFI (FAT-12/16/32)"),
					(0xf0, "Linux/PA-RISC bootloader"),
					(0xf1, "SpeedStor"),
					(0xf2, "DOS secondary"),
					(0xf4, "SpeedStor"),
					(0xf8, "EBBR protective"),
					(0xfb, "VMware VMFS"),
					(0xfc, "VMware VMKCORE"),
					(0xfd, "Linux raid auto"),
					(0xfe, "LANstep"),
					(0xff, "BBT"),
				]);
				let max_len = types.iter()
					.map(|(_, name)| name.len())
					.max()
					.unwrap_or(0);
				let term_width = 80; // TODO get from ioctl
				let entries_per_line = max(term_width / (max_len + 5), 1);

				for (i, (id, name)) in types.iter().enumerate() {
					print!("  {:02x} {:max_len$}", id, name);

					if i % entries_per_line == entries_per_line - 1 {
						println!();
					}
				}
			}

			Self::GPT => {
				// TODO
				todo!();
			}
		}
	}

	// TODO Return result instead
	/// Prompts for informations related to a new partition to be created.
	pub fn prompt_new_partition(&self) -> Partition {
		let (_extended, max_partition_count) = match self {
			Self::MBR => {
				// TODO get info from disk, to be passed as argument
				println!("Partition type");
				println!("   p   primary (TODO primary, TODO extended, TODO free)");
				println!("   e   extended (container for logical partitions)");

				let extended = prompt(Some("Select (default p): "), false)
					.map(|s| s == "e") // TODO handle invalid prompt (other than `p` and `e`)
					.unwrap_or(false);

				(extended, 4)
			}

			Self::GPT => (false, 128),
		};

		// Ask partition number
		let first = 1; // TODO get from disk
		let prompt_str = format!(
			"Partition number ({}-{}, default {}): ", first, max_partition_count, first
		);
		let partition_number = prompt(Some(&prompt_str), false)
			.map(|s| s.parse::<usize>())
			.transpose()
			.unwrap() // TODO handle error
			.unwrap_or(first);

		// Ask first sector
		let first_available = 2048; // TODO
		let last_available = 0; // TODO
		let prompt_str = format!(
			"First sector ({}-{}, default {})", first_available, last_available, first_available
		);
		let start = prompt(Some(&prompt_str), false)
			.map(|s| s.parse::<u64>())
			.transpose()
			.unwrap() // TODO handle error
			.unwrap_or(first_available);

		// Ask last sector
		let prompt_str = format!(
			"Last sector, +/-sectors or +/-size{{K,M,G,T,P}} ({}-{}, default {})",
			start, last_available, last_available
		);
		let end = prompt(Some(&prompt_str), false)
			.map(|s| {
				// TODO parse suffix
				s.parse::<u64>()
			})
			.transpose()
			.unwrap() // TODO handle error
			.unwrap_or(last_available);

		let sector_size = 512; // TODO get from disk?
		let size = (end - start) / sector_size as u64;

		Partition {
			start,
			size,

			part_type: "TODO".to_string(), // TODO

			uuid: None, // TODO

			bootable: false,
		}
	}
}

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
	pub fn serialize(dev: &Path, parts: &[Self]) -> String {
		let mut script = String::new();

		// Writing header
		// TODO label
		// TODO label-id
		script += format!("device: {}\n", dev.display()).as_str();
		script += "unit: sectors\n";
		script += "\n";

		// Writing partitions
		for (i, p) in parts.iter().enumerate() {
			script += &format!("{}{} : {}\n", dev.display(), i, p);
		}

		script
	}

	/// Deserializes a partitions list from a given sfdisk script.
	///
	/// Arguments:
	/// - `data` is the script.
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
