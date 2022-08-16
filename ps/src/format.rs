//! This module implement display formats.

use std::fmt;

/// Enumeration of data names.
pub enum Name {
	/// TODO doc
	Ruser,
	/// TODO doc
	User,
	/// TODO doc
	Rgroup,
	/// TODO doc
	Group,
	/// TODO doc
	Pid,
	/// TODO doc
	Ppid,
	/// TODO doc
	Pgid,
	/// TODO doc
	Pcpu,
	/// TODO doc
	Vsz,
	/// TODO doc
	Nice,
	/// TODO doc
	Etime,
	/// TODO doc
	Time,
	/// TODO doc
	Tty,
	/// TODO doc
	Comm,
	/// TODO doc
	Args,
}

impl Name {
	/// Returns the default display name.
	fn get_default_display(&self) -> &'static str {
		match self {
			Self::Ruser => "RUSER",
			Self::User => "USER",
			Self::Rgroup => "RGROUP",
			Self::Group => "GROUP",
			Self::Pid => "PID",
			Self::Ppid => "PPID",
			Self::Pgid => "PGID",
			Self::Pcpu => "%CPU",
			Self::Vsz => "VSZ",
			Self::Nice => "NI",
			Self::Etime => "ELAPSED",
			Self::Time => "TIME",
			Self::Tty => "TT",
			Self::Comm => "COMMAND",
			Self::Args => "COMMAND",
		}
	}
}

/// Structure representing a display format.
pub struct DisplayFormat {
	/// The list of names to be displayed along with their respective display name.
	pub names: Vec<(Name, String)>,
}

impl DisplayFormat {
	/// Tells whether the display format can be printed.
	pub fn can_print(&self) -> bool {
		self.names.iter()
			.filter(|(_, display_name)| !display_name.is_empty())
			.next()
			.is_some()
	}
}

impl Default for DisplayFormat {
	fn default() -> Self {
		Self {
			names: vec![
				(Name::Pid, Name::Pid.get_default_display().to_owned()),
				(Name::Tty, Name::Tty.get_default_display().to_owned()),
				(Name::Time, Name::Time.get_default_display().to_owned()),
				(Name::Comm, Name::Comm.get_default_display().to_owned()),
			],
		}
	}
}

impl fmt::Display for DisplayFormat {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		for (name, disp) in &self.names {
			if !disp.is_empty() {
				write!(fmt, " {}", disp)?;
			} else {
				// Add padding the same size as the default display name

				let len = name.get_default_display().len() + 1;
				for i in 0..len {
					write!(fmt, " ")?;
				}
			}
		}

		Ok(())
	}
}
