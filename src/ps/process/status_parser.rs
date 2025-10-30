/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! Process status parsing.

use super::Process;
use std::fs;
use std::io;

/// Parses the content of the file `/proc/{pid}/status`, where `{pid}` is the pid of the process.
pub struct StatusParser {
    /// The status file's content.
    status: String,
    /// The cmdline file's content.
    cmdline: String,
}

impl StatusParser {
    /// Creates a new instance for the given pid `pid`.
    pub fn new(pid: u32) -> io::Result<Self> {
        let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
        let cmdline = fs::read_to_string(format!("/proc/{pid}/cmdline"))?;
        Ok(Self { status, cmdline })
    }

    /// Creates a process structure from files.
    pub fn yield_process(self) -> Result<Process, ()> {
        let mut proc = Process::default();
        for line in self.status.split('\n') {
            if line.is_empty() {
                continue;
            }
            // Split the line to get the name and value
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
                }
                "gid" => {
                    let mut s = value.split_whitespace();
                    proc.gid = s.nth(0).ok_or(())?.parse::<u32>().map_err(|_| ())?;
                    proc.rgid = s.nth(2).ok_or(())?.parse::<u32>().map_err(|_| ())?;
                }
                // TODO tty
                _ => {}
            }
        }
        // Get full command line
        let mut cmdline = self
            .cmdline
            .chars()
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
