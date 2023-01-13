//! The `mkfs` tool allows to create a filesystem on a device.

use std::env;
use std::path::PathBuf;
use std::process::exit;

/// Structure storing command line arguments.
#[derive(Default)]
struct Args {
	/// The name of the current program used in command line.
	prog: String,
	/// The select filesystem type.
	fs_type: String,

	/// If true, print command line help.
	help: bool,

	/// The path to the device file on which the filesystem will be created.
	device_path: Option<PathBuf>,
}

fn parse_args() -> Args {
	let mut args: Args = Default::default();
	let mut iter = env::args();

	args.prog = iter.next().unwrap_or("mkfs".to_owned());

	let fs_type = if args.prog.contains('.') {
		args.prog.split('.').last()
	} else {
		None
	};
	args.fs_type = fs_type.unwrap_or("ext2").to_owned();

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"-h" | "--help" => args.help = true,

			// TODO implement other options
			// TODO get device path

			_ => {
				// TODO
				todo!();
			},
		}
	}

	args
}

fn main() {
	let args = parse_args();

	match args.fs_type.as_str() {
		"ext2" => {
			// TODO
			todo!();
		},
		// TODO

		_ => {
			eprintln!("{}: invalid filesystem type `{}`", args.prog, args.fs_type);
			exit(1);
		},
	}
}
