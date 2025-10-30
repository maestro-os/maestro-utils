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

//! Process structures.

mod status_parser;

use super::format::{DisplayFormat, Name};
use status_parser::StatusParser;
use std::fmt;
use std::fs;
use std::fs::ReadDir;
use std::io;

/// A process.
#[derive(Debug, Default)]
pub struct Process {
    /// The process's name.
    pub name: String,
    /// The full command.
    pub full_cmd: String,

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
    /// Returns an instance of [`ProcessDisplay`], used to display a process with the given `format`.
    pub fn display<'p, 'f>(&'p self, format: &'f DisplayFormat) -> ProcessDisplay<'p, 'f> {
        ProcessDisplay { proc: self, format }
    }
}

/// Display of a process's information.
pub struct ProcessDisplay<'p, 'f> {
    /// The process.
    proc: &'p Process,
    /// The display format.
    format: &'f DisplayFormat,
}

impl<'f, 'p> fmt::Display for ProcessDisplay<'f, 'p> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (name, _) in &self.format.0 {
            match name {
                Name::Ruser => write!(fmt, " {}", self.proc.ruid)?,
                Name::User => write!(fmt, " {}", self.proc.uid)?,
                Name::Rgroup => write!(fmt, " {}", self.proc.rgid)?,
                Name::Group => write!(fmt, " {}", self.proc.gid)?,
                Name::Pid => write!(fmt, " {}", self.proc.pid)?,
                Name::Ppid => write!(fmt, " {}", self.proc.ppid)?,
                // TODO Name::Pgid => write!(fmt, " {}", self.proc.pgid)?,
                // TODO Name::Pcpu => todo!(),
                // TODO Name::Vsz => todo!(),
                // TODO Name::Nice => todo!(),
                // TODO Name::Etime => todo!(),
                // TODO Name::Time => todo!(),
                Name::Tty => match &self.proc.tty {
                    Some(tty) => write!(fmt, " {tty}")?,
                    None => write!(fmt, " ?")?,
                },
                Name::Comm => write!(fmt, " {}", self.proc.name)?,
                Name::Args => write!(fmt, " {}", self.proc.full_cmd)?,
            }
        }
        Ok(())
    }
}

/// An iterator on the system's processes.
pub struct ProcessIterator(ReadDir);

impl ProcessIterator {
    /// Creates a new instance.
    pub fn new() -> io::Result<Self> {
        Ok(Self(fs::read_dir("/proc")?))
    }

    /// Returns the next PID in the iterator.
    /// If no PID is left, the function returns None.
    /// On error, the caller must retry.
    fn next_pid(&mut self) -> Option<Result<u32, ()>> {
        let entry = match self.0.next()? {
            Ok(e) => e,
            Err(_) => return Some(Err(())),
        };

        let file_name = entry.file_name().into_string();

        match file_name {
            Ok(file_name) => Some(file_name.parse::<u32>().map_err(|_| ())),
            Err(_) => Some(Err(())),
        }
    }

    /// Parses the status of process with PID `pid`.
    fn yield_proc(pid: u32) -> Result<Process, ()> {
        let status_parser = StatusParser::new(pid).map_err(|_| ())?;
        status_parser.yield_process()
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

            // Parsing process status
            match Self::yield_proc(pid) {
                Ok(proc) => return Some(proc),

                // On fail, try next process
                Err(_) => continue,
            }
        }
    }
}
