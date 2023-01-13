//! `fdisk` is an utility command used to manipulate disk partition tables.
//!
//! The `sfdisk` is also implemented in the same program, it has the purpose as `fdisk`, except it
//! uses scripting instead of prompting.

mod disk;

use disk::Disk;
use std::env;
use std::path::PathBuf;
use std::process::exit;
use utils::prompt::prompt;

/// Structure storing command line arguments.
#[derive(Default)]
struct Args {
	/// The name of the current program used in command line.
	prog: String,
	/// Tells whether the command is run in scripting mode.
	script: bool,

	/// If true, print command line help.
	help: bool,

	/// If true, list partitions instead of modifying the table.
	list: bool,

	/// The list of disk devices.
	disks: Vec<PathBuf>,
}

impl Args {
	/// Tells whether arguments are valid.
	fn is_valid(&self) -> bool {
		if self.help || self.list {
			return true;
		}

		self.disks.len() == 1
	}
}

fn parse_args() -> Args {
	let mut args: Args = Default::default();

	let mut iter = env::args();
	args.prog = iter.next().unwrap_or("fdisk".to_owned());
	args.script = args.prog.split('/').last() == Some("sfdisk");

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"-h" | "--help" => args.help = true,
			"-l" | "--list" => args.list = true,

			// TODO implement other options

			_ => args.disks.push(arg.into()),
		}
	}

	args
}

/// Prints command usage.
///
/// `prog` is the name of the current program.
fn print_usage(prog: &str) {
	eprintln!("{}: bad usage", prog);
	eprintln!("Try '{} --help' for more information.", prog);
}

/// Prints command help.
///
/// - `prog` is the name of the current program.
/// - `script` tells whether the program is run as `sfdisk`.
fn print_help(prog: &str, script: bool) {
	println!();
	println!("Usage:");
	println!(" {} [options] [disks...]", prog);
	println!();
	println!("Prints the list of partitions or modify it.");
	println!();
	println!("Options:");
	println!(" -h, --help\tPrints help.");
	println!(" -l, --list\tLists partitions.");
}

fn main() {
	let args = parse_args();

	if !args.is_valid() {
		print_usage(&args.prog);
		exit(1);
	}
	if args.help {
		print_help(&args.prog, args.script);
		exit(0);
	}

	if args.list {
		let disks_count = args.disks.len();

		for (i, path) in args.disks.into_iter().enumerate() {
			match Disk::read(path.clone()) {
				Ok(Some(disk)) => print!("{}", disk),

				Ok(None) => {
					eprintln!("{}: cannot open {}: Invalid argument", args.prog, path.display());
				},

				Err(e) => {
					eprintln!("{}: cannot open {}: {}", args.prog, path.display(), e);
				},
			}

			if i + 1 < disks_count {
				println!("\n");
			}
		}

		return;
	}

	if !args.script {
		let mut disk = Disk::read(args.disks[0].clone());

		while let Some(_cmd) = prompt(Some("Command (m for help): "), false) {
			// TODO execute commands
			todo!();
		}
		// TODO on exit without save, ask for confirm

		// TODO else on save, write table after confirm
	} else {
		// TODO Read and parse script
		// TODO Write partition table accordingly
		todo!();
	}
}
