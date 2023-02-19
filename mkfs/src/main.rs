//! The `mkfs` tool allows to create a filesystem on a device.

mod ext2;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use utils::prompt::prompt;

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
	/// Tells whether a filesystem corresponding to the factory is present on the given device
	/// `dev`.
	///
	/// `path` is the path to the device.
	fn is_present(&self, path: &Path, dev: &mut File) -> io::Result<bool>;

	/// Creates the filesystem on the given device `dev`.
	///
	/// `path` is the path to the device.
	fn create(&self, path: &Path, dev: &mut File) -> io::Result<()>;
}

fn main() {
	let args = parse_args();

	// TODO build factory according to arguments
	let factories = HashMap::<&str, Box<dyn FSFactory>>::from([
		("ext2", Box::new(ext2::Ext2Factory::default()) as Box<dyn FSFactory>),
	]);
	let factory = factories.get(args.fs_type.as_str()).unwrap_or_else(|| {
		eprintln!("{}: invalid filesystem type `{}`", args.prog, args.fs_type);
		exit(1);
	});

	let device_path = args.device_path.unwrap_or_else(|| {
		eprintln!("{}: specify path to a device", args.prog);
		exit(1);
	});

	let mut file = OpenOptions::new()
		.write(true)
		.open(&device_path)
		.unwrap_or_else(|e| {
			eprintln!("{}: {}: {}", args.prog, device_path.display(), e);
			exit(1);
		});

	let prev_fs = factories.iter()
		.filter(|(_, factory)| {
			factory.is_present(&device_path, &mut file).unwrap_or_else(|e| {
				eprintln!("{}: {}: {}", args.prog, device_path.display(), e);
				exit(1);
			})
		})
		.next();
	if let Some((prev_fs_type, _prev_fs_factory)) = prev_fs {
		println!("{} contains a file system of type: {}", device_path.display(), prev_fs_type);
		// TODO print details on fs (use factory)

		let confirm = prompt(Some("Proceed anyway? (y/N) "), false)
			.map(|s| s.to_lowercase() == "y")
			.unwrap_or(false);
		if !confirm {
			eprintln!("Abort.");
			exit(1);
		}
	}

	factory.create(&device_path, &mut file).unwrap_or_else(|e| {
		eprintln!("{}: failed to create filesystem: {}", args.prog, e);
		exit(1);
	});
}
