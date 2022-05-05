//! The passwd, shadow and group files are mainly used to store respectively the users list, the
//! passwords list and the groups list.

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Write;

/// The path to the passwd file.
pub const PASSWD_PATH: &str = "/etc/passwd";
/// The path to the shadow file.
pub const SHADOW_PATH: &str = "/etc/shadow";
/// The path to the group file.
pub const GROUP_PATH: &str = "/etc/group";

// TODO For each files, use a backup file with the same path but with `-` appended at the end

/// Structure representing a user. This entry is present in the passwd file.
pub struct User {
	/// The user's login name.
	pub login_name: String,
	/// The user's encrypted password. If `x`, the password is located in the shadow file.
	pub password: String,
	/// The user ID.
	pub uid: u32,
	/// The user's group ID.
	pub gid: u32,
	/// User comment.
	pub comment: String,
	/// User's home path.
	pub home: String,
	/// User's command interpreter.
	pub interpreter: String,
}

/// Structure representing a shadow entry.
pub struct Shadow {
	/// The user's login name.
	pub login_name: String,
	/// The user's encrypted password.
	pub password: String,
	/// The date of the last password change in number of days since the Unix Epoch.
	pub last_change: u32,
	/// The minimum number of days to wait before the user becomes usable.
	pub minimum_age: u32,
	/// The maximum number of days to the password is valid. If this delay is exceeded, the user
	/// will be asked to change her password next time she logs.
	pub maximum_age: u32,
	/// The number of days before the password expires during which the user will be warned to
	/// change her password.
	pub warning_period: u32,
	/// The number of days after password expiration during which the user can still use her
	/// password. Passed this delay, she will have to contact her system administrator.
	pub inactivity_period: u32,
	/// The number of days after the Unix Epoch after which login to the user account will be
	/// denied.
	pub account_expiration: u32,
	/// Reserved field.
	pub reserved: String,
}

/// Structure representing a group.
pub struct Group {
	/// The group's name.
	pub group_name: String,
	/// The encrypted group's password.
	pub password: String,
	/// The group's ID.
	pub gid: String,
	/// The list of users member of this group, comma-separated.
	pub users_list: String,
}

/// Reads and parses the file at path `path`.
pub fn parse_user_file(path: &str) -> Result<Vec<Vec<String>>> {
	let file = File::open(path)?;
	let mut data = vec![];

	for l in BufReader::new(file).lines() {
		data.push(l?.split(":").map(| s | s.to_owned()).collect::<_>());
	}

	Ok(data)
}

/// Writes the file at path `path` with data `data`.
pub fn write(path: &str, data: &Vec<Vec<String>>) -> Result<()> {
	let mut file = File::open(path)?;
	let mut content = String::new();

	for line in data {
		for (i, elem) in line.iter().enumerate() {
			if elem.contains(':') {
				return Err(Error::new(ErrorKind::Other, "entry cannot contain character `:`"));
			}

			content += elem;
			if i + 1 < line.len() {
				content += ":";
			}
		}
	}

	file.write(content.as_bytes())?;
	Ok(())
}
