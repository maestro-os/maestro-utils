//! Module implementing process structures.

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
	// TODO
}

impl ProcessIterator {
	/// Creates a new instance.
	pub fn new() -> Self {
		Self {
			// TODO
		}
	}
}

impl Iterator for ProcessIterator {
	type Item = Process;

	fn next(&mut self) -> Option<Self::Item> {
		// TODO
		todo!();
	}
}
