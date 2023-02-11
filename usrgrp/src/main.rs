//! The `usrgrp` command implements the following commands:
//! - `useradd`: create a new user
//! - `usermod`: modify an user
//! - `userdel`: delete an user
//! - `groupadd`: create a new group
//! - `groupmod`: modify a group
//! - `groupdel`: delete a group

use std::env;
use std::process::exit;

/// Command line arguments.
pub enum Args {
	UserAdd {
		/// If set, display usage.
		help: bool,

		/// The home directory of the new user.
		home_dir: Option<String>,

		/// The expiry timestamp for the user.
		expire_ts: Option<u64>,
		/// The inactivity period of the new user.
		inactive_period: Option<u64>,

		/// If set, create the user's home directory.
		create_home: bool,

		/// If set, create a group with the same name as the user.
		user_group: bool,

		/// The UID for the new user.
		uid: Option<u16>,
		/// The ID or name of the group for the new user.
		gid: Option<String>,

		/// The encrypted password for the new password.
		password: Option<String>,
		/// The login shell of the new account.
		shell: Option<String>,

		/// The username.
		name: String,
	},

	UserMod {
		/// If set, display usage.
		help: bool,

		// TODO

		/// The username.
		name: String,
	},

	UserDel {
		/// If set, display usage.
		help: bool,

		/// If set, delete the user even if still logged in.
		force: bool,

		/// If set, remove the home directory and mail spool.
		remove_home: bool,

		/// The username.
		name: String,
	},

	GroupAdd {
		/// If set, display usage.
		help: bool,

		/// The ID to use for the group.
		gid: Option<u16>,

		/// The group name.
		name: String,
	},

	GroupMod {
		/// If set, display usage.
		help: bool,

		// TODO

		/// The group name.
		name: String,
	},

	GroupDel {
		/// If set, display usage.
		help: bool,

		/// If set, delete the group even if it is the primary group of a user.
		force: bool,

		/// The group name.
		name: String,
	},
}

/// Parses command line arguments.
fn parse_args() -> Args {
	let mut args_iter = env::args();

	let bin = match args_iter.next() {
		Some(bin) => bin,

		None => {
			// TODO return usage
			todo!();
		}
	};

	match bin.as_str() {
		"useradd" => {
			// TODO
			todo!();
		}

		"usermod" => {
			// TODO
			todo!();
		}

		"userdel" => {
			// TODO
			todo!();
		}

		"groupadd" => {
			// TODO
			todo!();
		}

		"groupmod" => {
			// TODO
			todo!();
		}

		"groupdel" => {
			// TODO
			todo!();
		}

		_ => exit(1),
	}
}

fn main() {
	let args = parse_args();

	match args {
		Args::UserAdd { .. } => {
			// TODO
			todo!();
		},

		Args::UserMod { .. } => {
			// TODO
			todo!();
		},

		Args::UserDel { .. } => {
			// TODO
			todo!();
		},

		Args::GroupAdd { .. } => {
			// TODO
			todo!();
		},

		Args::GroupMod { .. } => {
			// TODO
			todo!();
		},

		Args::GroupDel { .. } => {
			// TODO
			todo!();
		},
	}
}
