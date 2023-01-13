//! The `mkfs` tool allows to create a filesystem on a device.

mod ext2;

use std::env;
use std::fs::File;
use std::io;
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
				// TODO handle case when several devices are given
				args.device_path = Some(PathBuf::from(arg));
			},
		}
	}

	args
}

/// A trait representing an object used to create a filesystem on a device.
pub trait FSFactory {
	/// Creates the filesystem on the given device `dev`.
	///
	/// `dev` is the file of the device on which the filesystem will be created.
	fn create(&self, dev: &mut File) -> io::Result<()>;
}

fn main() {
	let args = parse_args();

	let factory = match args.fs_type.as_str() {
		"ext2" => ext2::Ext2Factory {},
		// TODO

		_ => {
			eprintln!("{}: invalid filesystem type `{}`", args.prog, args.fs_type);
			exit(1);
		},
	};

	let device_path = args.device_path.unwrap_or_else(|| {
		eprintln!("{}: specify path to a device", args.prog);
		exit(1);
	});

	let mut file = File::open(&device_path).unwrap_or_else(|e| {
		eprintln!("{}: cannot open device `{}`: {}", args.prog, device_path.display(), e);
		exit(1);
	});

	// TODO detect filesystem on the device. If one is present, ask for confirmation

	if let Err(e) = factory.create(&mut file) {
		eprintln!("{}: failed to create filesystem: {}", args.prog, e);
		exit(1);
	}
}
