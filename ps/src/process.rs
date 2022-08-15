//! Module implementing process structures.

use std::fs::ReadDir;
use std::fs;

/// Structure representing a process.
pub struct Process {
	/// The process's PID.
	pub pid: u32,
	/// The process's user ID.
	pub uid: u32,
	/// The process's real user ID.
	pub ruid: u32,
	/// The process's group ID.
	pub gid: u32,
	/// The process's real group ID.
	pub rgid: u32,
	/// The process's TTY.
	pub tty: Option<String>,
}

/// An iterator on the system's processes.
pub struct ProcessIterator {
	/// The iterator on procfs files.
	files: ReadDir,
}

impl ProcessIterator {
	/// Creates a new instance.
	pub fn new() -> Self {
		Self {
			files: fs::read_dir("/proc").unwrap(), // TODO Handle error properly
		}
	}
}

impl Iterator for ProcessIterator {
	type Item = Process;

	fn next(&mut self) -> Option<Self::Item> {
		/*let _pid = self.files.map(| _entry | {
				// TODO Get entry names
				todo!();
			})
			.filter_map(| _name | {
				// TODO Filter non-processes and map PIDs to integers
				Some(0u32)
			})
			.next()?;*/

		// TODO Retrieve informations from `/proc/{pid}/status`
		todo!();
	}
}
