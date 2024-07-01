//! `su` is a command allowing to run another command with a substitute user and group ID.

use std::env;
use std::process::exit;
use std::process::Command;

use utils::prompt::prompt;

/// The command's arguments.
#[derive(Default)]
struct Args<'s> {
    /// The user which executes the command. If None, using root.
    user: Option<&'s str>,
    /// The group which executes the command. If None, using root.
    group: Option<&'s str>,

    /// The shell to execute. If None, using the default.
    shell: Option<&'s str>,

    /// Arguments for the command to execute.
    args: Vec<&'s str>,
}

/// Parses the given CLI arguments `args` and returns their representation in the `Args` structure.
fn parse_args(args: &Vec<String>) -> Args<'_> {
    let mut result = Args::default();
    // Iterating on arguments, skipping binary's name
    let mut iter = args.iter().skip(1).peekable();

    // Tells whether arguments contain initial options
    let has_options = {
        iter.peek()
            .map(|first_arg| {
                first_arg
                    .chars()
                    .peekable()
                    .peek()
                    .map(|first_char| *first_char == '-')
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

    result.user = iter.next().map(|s| s.as_str());
    result.args = iter.map(|s| s.as_str()).collect();

    result
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let args = parse_args(&args);

    let _user = args.user.unwrap_or("root");
    // TODO Read user's entry
    let shell = args.shell.unwrap_or("TODO");

    let _pass = prompt("Password: ", true);
    let correct = false; // TODO Check password against user's

    if correct {
        // TODO Change user

        // Running the shell
        let status = Command::new(&shell)
            .args(args.args)
            // TODO Set env
            .status()
            .unwrap_or_else(|_| {
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
