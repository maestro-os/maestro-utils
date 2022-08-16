//! Module implementing process structures.

mod status_parser;

use crate::format::DisplayFormat;
use status_parser::StatusParser;
use std::fmt;
use std::fs::ReadDir;
use std::fs;

/// Structure representing a process.
#[derive(Default)]
pub struct Process {
	/// The process's name.
	pub name: String,

	/// The process's PID.
	pub pid: u32,
	/// The PID of the process's parent.
	pub ppid: u32,

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

impl Process {
	/// Returns an instance of ProcessDisplay, used to display a process with the given format.
	pub fn display<'p, 'f>(&'p self, format: &'f DisplayFormat) -> ProcessDisplay<'p, 'f> {
		ProcessDisplay {
			proc: self,
			format,
		}
	}
}

/// Structure used to display a process's informations.
pub struct ProcessDisplay<'p, 'f> {
	/// The process.
	proc: &'p Process,
	/// The display format.
	format: &'f DisplayFormat,
}

impl<'f, 'p> fmt::Display for ProcessDisplay<'f, 'p> {
	fn fmt(&self, _fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		// TODO
		Ok(())
	}
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
			files: fs::read_dir("/proc").unwrap() // TODO Handle error properly
		}
	}

	/// Returns the next PID in the iterator.
	/// If no PID is left, the function returns None.
	/// On error, the caller must retry.
	fn next_pid(&mut self) -> Option<Result<u32, ()>> {
		let entry = match self.files.next()? {
			Ok(e) => e,
			Err(_) => return Some(Err(())),
		};

		let file_name = entry.file_name().into_string();

		match file_name {
			Ok(file_name) => Some(file_name.parse::<u32>().map_err(| _ | ())),
			Err(_) => Some(Err(())),
		}
	}
}

impl Iterator for ProcessIterator {
	type Item = Process;

	fn next(&mut self) -> Option<Self::Item> {
		// Looping until finding a valid process or reaching the end
		loop {
			// Getting the next PID
			let pid = match self.next_pid()? {
				Ok(pid) => pid,
				Err(_) => continue,
			};

			// The path to the process's status file
			let path = format!("/proc/{}/status", pid);
			// Reading the process's status file
			let content = match fs::read_to_string(path) {
				Ok(content) => content,
				Err(_) => continue,
			};

			// Parsing process status
			let status_parser = StatusParser::new(&content);
			match status_parser.yield_process() {
				Ok(proc) => return Some(proc),

				// On fail, try next process
				Err(_) => continue,
			}
		}
	}
}
