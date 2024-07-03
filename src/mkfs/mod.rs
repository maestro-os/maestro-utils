//! The `mkfs` tool allows to create a filesystem on a device.

mod ext2;

use std::collections::HashMap;
use std::env::ArgsOs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::process::exit;
use utils::error;
use utils::prompt::prompt;

/// Structure storing command line arguments.
#[derive(Default)]
struct Args {
    /// The select filesystem type.
    fs_type: String,
    /// If true, print command line help.
    help: bool,
    /// The path to the device file on which the filesystem will be created.
    device_path: Option<PathBuf>,
}

fn parse_args(args: ArgsOs) -> Args {
    let mut res: Args = Default::default();
    for arg in args {
        match arg.to_str() {
            Some("-h" | "--help") => res.help = true,
            // TODO implement other options
            // TODO get device path
            _ => {
                // TODO handle case when several devices are given
                res.device_path = Some(PathBuf::from(arg));
            }
        }
    }
    res
}

/// A trait representing an object used to create a filesystem on a device.
pub trait FSFactory {
    /// Tells whether a filesystem corresponding to the factory is present on the given device
    /// `dev`.
    fn is_present(&self, dev: &mut File) -> io::Result<bool>;

    /// Creates the filesystem on the given device `dev`.
    fn create(&self, dev: &mut File) -> io::Result<()>;
}

pub fn main(fs_name: &str, args: ArgsOs) {
    let args = parse_args(args);

    // TODO build factory according to arguments
    let factories = HashMap::<&str, Box<dyn FSFactory>>::from([(
        "ext2",
        Box::<ext2::Ext2Factory>::default() as Box<dyn FSFactory>,
    )]);
    let factory = factories.get(args.fs_type.as_str()).unwrap_or_else(|| {
        error(
            "mkfs",
            format_args!("invalid filesystem type `{}`", args.fs_type),
        );
    });
    let device_path = args.device_path.unwrap_or_else(|| {
        error("mkfs", "specify path to a device");
    });
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&device_path)
        .unwrap_or_else(|e| {
            error("mkfs", format_args!("{}: {e}", device_path.display()));
        });

    let prev_fs = factories.iter().find(|(_, factory)| {
        factory.is_present(&mut file).unwrap_or_else(|e| {
            error("mkfs", format_args!("{}: {e}", device_path.display()));
        })
    });
    if let Some((prev_fs_type, _prev_fs_factory)) = prev_fs {
        println!(
            "{} contains a file system of type: {prev_fs_type}",
            device_path.display()
        );
        // TODO print details on fs (use factory)

        let confirm = prompt("Proceed anyway? (y/N) ", false)
            .map(|s| s.to_lowercase() == "y")
            .unwrap_or(false);
        if !confirm {
            eprintln!("Abort.");
            exit(1);
        }
    }
    factory.create(&mut file).unwrap_or_else(|e| {
        error("mkfs", format_args!("failed to create filesystem: {e}"));
    });
}
