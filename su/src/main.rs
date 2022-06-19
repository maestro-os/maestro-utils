//! `su` is a command allowing to run an other command with a substitute user and group ID.

use std::env;
use std::process::Command;
use std::process::exit;

use utils::prompt::prompt;

/// Structure representing the command's arguments.
#[derive(Default)]
struct Args {
	/// The user which executes the command. If None, using root.
	user: Option<String>,
	/// The group which executtes the command. If None, using root.
	group: Option<String>,

	/// The shell to execute. If None, using the default.
	shell: Option<String>,

	/// Arguments for the command to execute.
	args: Vec<String>,
}

/// Parses the given CLI arguments `args` and returns their representation in the `Args` structure.
fn parse_args(args: Vec<String>) -> Args {
	let mut result = Args::default();
	// Iterating on arguments, skipping binary's name
	let mut iter = args.iter().skip(1).peekable();

	// Tells whether arguments contain initial options
	let has_options = {
		iter.peek()
			.map(| first_arg | {
				first_arg.chars().peekable().peek()
					.map(| first_char | *first_char == '-')
					.unwrap_or(false)
			})
			.unwrap_or(false)
	};

	// Parsing options if present
	if has_options {
		while let Some(a) = iter.next() {
			if a == "-" {
				break;
			}

			// TODO
		}
	}

	result.user = iter.next().map(| s | s.clone());
	result.args = iter.map(| s | s.clone()).collect::<Vec<String>>();

	result
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let args = parse_args(args);

	let _user = args.user.unwrap_or("root".to_owned());
	// TODO Read user's entry
	let shell = args.shell.unwrap_or("TODO".to_owned());

	let _pass = prompt(None, true);
	let correct = false; // TODO Check password against user's

	if correct {
		// TODO Change user

		// Running the shell
		let status = Command::new(&shell)
			.args(args.args)
			// TODO Set env
			.status()
			.unwrap_or_else(| _ | {
				eprintln!("su: Failed to run shell `{}`", shell);
				exit(1);
			});

		// Exiting with the shell's status
		exit(status.code().unwrap());
	} else {
		eprintln!("su: Authentication failure");
		exit(1);
	}
}
