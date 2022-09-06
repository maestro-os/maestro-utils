//! This module implements a parser for the status of a process.

use std::fs;
use std::io;
use super::Process;

/// The status parser parses the content of the file `/proc/{pid}/status`, where `{pid}` is the pid
/// of the process.
pub struct StatusParser {
	/// The status file's content.
	status_content: String,
	/// The cmdline file's content.
	cmdline_content: String,
}

impl StatusParser {
	/// Creates a new instance for the given pid `pid`.
	pub fn new(pid: u32) -> Result<Self, io::Error> {
		// Reading the process's status file
		let status_content = fs::read_to_string(format!("/proc/{}/status", pid))?;

		// Reading the process's status file
		let cmdline_content = fs::read_to_string(format!("/proc/{}/cmdline", pid))?;

		Ok(Self {
			status_content,
			cmdline_content,
		})
	}

	/// Creates a process structure from files.
	pub fn yield_process(self) -> Result<Process, ()> {
		let mut proc = Process::default();

		for line in self.status_content.split('\n') {
			if line.is_empty() {
				continue;
			}

			// Splitting the line to get the name and value
			let (name, value) = line.find(':').map(|i| line.split_at(i)).ok_or(())?;
			let name = name.to_lowercase();
			let value = value[1..].trim();

			match name.as_str() {
				"name" => proc.name = value.to_string(),

				"pid" => proc.pid = value.parse::<u32>().map_err(|_| ())?,
				"ppid" => proc.ppid = value.parse::<u32>().map_err(|_| ())?,

				"uid" => {
					let mut s = value.split_whitespace();

					proc.uid = s.nth(0).ok_or(())?.parse::<u32>().map_err(|_| ())?;
					proc.ruid = s.nth(2).ok_or(())?.parse::<u32>().map_err(|_| ())?;
				},
				"gid" => {
					let mut s = value.split_whitespace();

					proc.gid = s.nth(0).ok_or(())?.parse::<u32>().map_err(|_| ())?;
					proc.rgid = s.nth(2).ok_or(())?.parse::<u32>().map_err(|_| ())?;
				},

				// TODO tty

				_ => {},
			}
		}

		// Getting full command line
		let mut cmdline = self.cmdline_content.chars()
			.map(|c| match c {
				'\0' => ' ',
				_ => c,
			})
			.collect::<String>();
		cmdline.pop();
		proc.full_cmd = cmdline;

		Ok(proc)
	}
}
