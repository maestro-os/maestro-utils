//! This module implements a parser for the status of a process.

use super::Process;

/// The status parser parses the content of the file `/proc/{pid}/status`, where `{pid}` is the pid
/// of the process.
pub struct StatusParser<'a> {
	/// The file's content.
	content: &'a str,
}

impl<'a> StatusParser<'a> {
	/// Creates a new instance with the given file content.
	pub fn new(content: &'a str) -> Self {
		Self {
			content,
		}
	}

	/// Creates a process structure using the content of the file.
	pub fn yield_process(self) -> Result<Process, ()> {
		let mut proc = Process::default();

		for line in self.content.split('\n') {
			// Splitting the line to get the name and value
			let mut s = line.split(':');
			let name = s.next().ok_or(())?.to_lowercase();
			let value = s.next().ok_or(())?.trim();
			if s.next().is_some() {
				return Err(());
			}

			match name.as_str() {
				"name" => proc.name = value.to_string(),

				"pid" => proc.pid = value.parse::<u32>().map_err(|_| ())?,
				"ppid" => proc.ppid = value.parse::<u32>().map_err(|_| ())?,

				"uid" => {
					let mut s = value.split_whitespace();

					proc.uid = s.nth(0).ok_or(())?.parse::<u32>().map_err(|_| ())?;
					proc.ruid = s.nth(3).ok_or(())?.parse::<u32>().map_err(|_| ())?;
				},
				"gid" => {
					let mut s = value.split_whitespace();

					proc.gid = s.nth(0).ok_or(())?.parse::<u32>().map_err(|_| ())?;
					proc.rgid = s.nth(3).ok_or(())?.parse::<u32>().map_err(|_| ())?;
				},

				// TODO tty

				_ => {},
			}
		}

		Ok(proc)
	}
}
