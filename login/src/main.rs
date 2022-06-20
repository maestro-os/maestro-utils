//! `login` prompts a username/password to authenticate on a new session.

use std::collections::HashMap;
use std::env;
use std::ffi::OsString;
use std::process::Command;
use std::process::exit;
use std::time::Duration;
use utils::prompt::prompt;
use utils::user::User;
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
		let user_entry = passwd.into_iter()
			.filter(| e | e.login_name == login)
			.next();

		util::exec_wait(Duration::from_millis(1000), || {
			if let Some(user_entry) = user_entry {
				// Checking password against user entry
				let correct = user_entry.check_password(&pass)
					.unwrap_or_else(|| {
						if let Some(shadow) = shadow {
							shadow.into_iter()
								.filter(| e | e.login_name == login)
								.map(| e | e.check_password(&pass))
								.next()
								.unwrap_or(false)
						} else {
							false
						}
					});

				if correct {
					let User {
						uid,
						gid,
						home,
						interpreter,
						..
					} = user_entry;

					// Changing user
					user::set(uid, gid).unwrap_or_else(| e | {
						eprintln!("{}", e);
						exit(1);
					});

					let mut env = env::vars_os().collect::<HashMap<OsString, OsString>>();
					env.insert("HOME".into(), home.into());

					// Running the user's program
					let status = Command::new(&interpreter)
						.envs(env)
						.status()
						.unwrap_or_else(| _ | {
							eprintln!("login: Failed to run shell `{}`", interpreter);
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
