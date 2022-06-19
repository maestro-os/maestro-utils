//! `login` prompts a username/password to authenticate on a new session.

use std::env;
use std::process::Command;
use std::process::exit;
use std::time::Duration;
use utils::prompt::prompt;
use utils::user;
use utils::util;

fn main() {
	let _args: Vec<String> = env::args().collect(); // TODO Parse and use

	loop {
		println!();

		// Getting user prompt
		let hostname = "TODO"; // TODO Get hostname
		let user_prompt = format!("{} login: ", hostname);

		// Prompting login and password
		let login = prompt(Some(&user_prompt), false).unwrap_or_else(|| exit(1));
		let pass = prompt(None, true).unwrap_or_else(|| exit(1));

		// Reading users lists
		let passwd = user::read_passwd(user::PASSWD_PATH).unwrap_or_else(| _ | {
			eprintln!("Cannot read passwd file!");
			exit(1);
		});
		let shadow = user::read_shadow(user::SHADOW_PATH).ok();

		// Getting user from prompted login
		let user_entry = passwd.iter()
			.filter(| e | e.login_name == login)
			.next();

		util::exec_wait(Duration::from_millis(1000), || {
			if let Some(user_entry) = user_entry {
				// Checking password against user entry
				let correct = user_entry.check_password(&pass)
					.unwrap_or_else(|| {
						if let Some(shadow) = shadow {
							println!("A");
							shadow.iter()
								.filter(| e | e.login_name == login)
								.map(| e | e.check_password(&pass))
								.next()
								.unwrap_or(false)
						} else {
							false
						}
					});

				if correct {
					// Changing user
					user::set(user_entry.uid, user_entry.gid).unwrap_or_else(| e | {
						eprintln!("{}", e);
						exit(1);
					});

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
		});

		eprintln!("Login incorrect");
	}
}
