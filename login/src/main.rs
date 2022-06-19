//! `login` prompts a username/password to authenticate on a new session.

use std::env;
use std::process::Command;
use std::process::exit;
use utils::prompt::prompt;
use utils::user;

fn main() {
	let _args: Vec<String> = env::args().collect(); // TODO Parse and use

	loop {
		println!();

		// Getting user prompt
		let hostname = "TODO"; // TODO Get hostname
		let user_prompt = format!("{} login: ", hostname);

		// Prompting login and password
		let login = prompt(Some(&user_prompt), false);
		let pass = prompt(None, true);

		// Reading users lists
		let passwd = user::read_passwd(user::PASSWD_PATH).unwrap_or_else(| _ | {
			eprintln!("Cannot read passwd file!");
			exit(1);
		});
		let shadow = user::read_shadow(user::SHADOW_PATH).unwrap_or_else(| _ | {
			eprintln!("Cannot read shadow file!");
			exit(1);
		});

		// Getting user from prompted login
		let user_entry = passwd.iter()
			.filter(| e | e.login_name == login)
			.next();

		if let Some(user_entry) = user_entry {
			// Checking password against user entry
			let correct = user_entry.check_password(&pass)
				.unwrap_or_else(|| {
					shadow.iter()
						.filter(| e | e.login_name == login)
						.next()
						.map(| e | e.check_password(&pass))
						.unwrap_or(false)
				});

			if correct {
				// TODO Change user

				// Running the shell
				let status = Command::new(&user_entry.interpreter)
					// TODO Set env
					.status()
					.unwrap_or_else(| _ | {
						eprintln!("login: Failed to run shell `{}`", user_entry.interpreter);
						exit(1);
					});

				// Exiting with the shell's status
				exit(status.code().unwrap());
			}
		}

		eprintln!("Login incorrect");
	}
}
