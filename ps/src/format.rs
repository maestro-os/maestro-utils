//! This module implement display formats.

use std::fmt;

/// Enumeration of data names.
pub enum Name {
    /// The real user ID.
    Ruser,
    /// The effective user ID.
    User,
    /// The real group ID.
    Rgroup,
    /// The effective group ID.
    Group,
    /// The process ID.
    Pid,
    /// The parent process ID.
    Ppid,
    ///// The process group ID.
    //Pgid,
    ///// TODO doc
    //Pcpu,
    ///// TODO doc
    //Vsz,
    ///// The nice value.
    //Nice,
    ///// TODO doc
    //Etime,
    ///// TODO doc
    //Time,
    /// The terminal.
    Tty,
    /// The name.
    Comm,
    /// The command line arguments.
    Args,
}

impl Name {
    /// Returns the variant associated with the given string.
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ruser" => Some(Self::Ruser),
            "user" => Some(Self::User),
            "rgroup" => Some(Self::Rgroup),
            "group" => Some(Self::Group),
            "pid" => Some(Self::Pid),
            "ppid" => Some(Self::Ppid),
            // TODO "pgid" => Some(Self::Pgid),
            // TODO "pcpu" => Some(Self::Pcpu),
            // TODO "vsz" => Some(Self::Vsz),
            // TODO "nice" => Some(Self::Nice),
            // TODO "etime" => Some(Self::Etime),
            // TODO "time" => Some(Self::Time),
            "tty" => Some(Self::Tty),
            "comm" => Some(Self::Comm),
            "args" => Some(Self::Args),
            _ => None,
        }
    }

    /// Returns the default display name.
    fn get_default_display(&self) -> &'static str {
        match self {
            Self::Ruser => "RUSER",
            Self::User => "USER",
            Self::Rgroup => "RGROUP",
            Self::Group => "GROUP",
            Self::Pid => "PID",
            Self::Ppid => "PPID",
            // TODO Self::Pgid => "PGID",
            // TODO Self::Pcpu => "%CPU",
            // TODO Self::Vsz => "VSZ",
            // TODO Self::Nice => "NI",
            // TODO Self::Etime => "ELAPSED",
            // TODO Self::Time => "TIME",
            Self::Tty => "TT",
            Self::Comm | Self::Args => "COMMAND",
        }
    }
}

/// A display format.
pub struct DisplayFormat(pub Vec<(Name, String)>);

impl DisplayFormat {
    /// Tells whether the display format can be printed.
    pub fn can_print(&self) -> bool {
        self.0
            .iter()
            .any(|(_, display_name)| !display_name.is_empty())
    }
}

impl Default for DisplayFormat {
    fn default() -> Self {
        Self(vec![
            (Name::Pid, Name::Pid.get_default_display().to_owned()),
            (Name::Tty, Name::Tty.get_default_display().to_owned()),
            // TODO (Name::Time, Name::Time.get_default_display().to_owned()),
            (Name::Comm, Name::Comm.get_default_display().to_owned()),
        ])
    }
}

impl fmt::Display for DisplayFormat {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (name, disp) in &self.0 {
            if !disp.is_empty() {
                write!(fmt, " {disp}")?;
            } else {
                // Add padding the same size as the default display name
                let len = name.get_default_display().len() + 1;
                write!(fmt, "{:<len$}", "")?;
            }
        }
        Ok(())
    }
}

/// Parses the given display format.
pub fn parse_display_format(s: &str) -> Result<DisplayFormat, ()> {
    let names = s
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .map(|s| {
            // Split name and display name
            let split = s.find('=').map(|i| {
                let (name, display_name) = s.split_at(i);
                (name, &display_name[1..])
            });
            if let Some((name, display_name)) = split {
                let name = Name::from_str(name).ok_or(())?;
                Ok((name, display_name.to_owned()))
            } else {
                let name = Name::from_str(s).ok_or(())?;
                let display_name = name.get_default_display();
                Ok((name, display_name.to_owned()))
            }
        })
        .collect::<Result<_, ()>>()?;
    Ok(DisplayFormat(names))
}
