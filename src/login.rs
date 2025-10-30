/*
 * Copyright 2025 Luc Lenôtre
 *
 * This file is part of Maestro.
 *
 * Maestro is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Maestro is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Maestro. If not, see <https://www.gnu.org/licenses/>.
 */

//! `login` prompts a username/password to authenticate on a new session.

use std::env::ArgsOs;
use std::ffi::{CString, OsString};
use std::fmt::Formatter;
use std::os::unix::ffi::OsStrExt;
use std::process::exit;
use std::ptr::null;
use std::time::Duration;
use std::{env, fmt, fs, io, iter};
use utils::prompt::prompt;
use utils::user;
use utils::user::{PASSWD_PATH, SHADOW_PATH, Shadow, User};
use utils::util;
use utils::util::get_hostname;

/// Builds an environment variable in the form: name=value
fn build_env_var(name: &str, value: impl IntoIterator<Item = u8>) -> CString {
    let data: Vec<u8> = name
        .as_bytes()
        .iter()
        .cloned()
        .chain(iter::once(b'='))
        .chain(value)
        .chain(iter::once(b'\0'))
        .collect();
    CString::from_vec_with_nul(data).unwrap()
}

/// Switches to the given user after login is successful.
///
/// Arguments:
/// - `logname` is the name of the user used to log in.
/// - `user` is the user to switch to.
fn switch_user(logname: &str, user: User) -> io::Result<!> {
    let User {
        login_name,
        uid,
        gid,
        home,
        interpreter,
        ..
    } = user;
    // Prepare environment
    let term = env::var_os("TERM").unwrap_or_else(|| {
        // TODO fetch from the terminal
        "linux".into()
    });
    let shell = match interpreter {
        "" => "/bin/sh",
        i => i,
    };
    let path = match uid {
        0 => "/usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin",
        _ => "/usr/local/bin:/bin:/usr/bin",
    };
    let mail = "/var/spool/mail/".bytes().chain(login_name.bytes());
    // Prepare `execve` arguments
    let bin = CString::new(shell).unwrap(); // TODO handle error?
    let argv = [bin.as_ptr(), null()];
    let env_home = build_env_var("HOME", home.as_os_str().as_bytes().iter().cloned());
    let env_user = build_env_var("USER", login_name.bytes());
    let env_logname = build_env_var("LOGNAME", logname.bytes());
    let env_term = build_env_var("TERM", term.as_bytes().iter().cloned());
    let env_shell = build_env_var("SHELL", shell.bytes());
    let env_path = build_env_var("PATH", path.bytes());
    let env_mail = build_env_var("MAIL", mail);
    let envp = [
        env_home.as_ptr(),
        env_user.as_ptr(),
        env_logname.as_ptr(),
        env_term.as_ptr(),
        env_shell.as_ptr(),
        env_path.as_ptr(),
        env_mail.as_ptr(),
        null(),
    ];
    // Set current user
    user::set(uid, gid)?;
    // Set current working directory
    env::set_current_dir(home)?;
    // Execute interpreter
    let res = unsafe { libc::execve(bin.as_ptr(), argv.as_ptr(), envp.as_ptr()) };
    if res >= 0 {
        // In theory, `execve` will never return when successful
        unreachable!();
    } else {
        Err(io::Error::last_os_error())
    }
}

/// The login prompt.
struct LoginPrompt(OsString);

impl fmt::Display for LoginPrompt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} login: ", self.0.display())
    }
}

pub fn main(_args: ArgsOs) {
    let login_prompt = LoginPrompt(get_hostname());
    loop {
        println!();
        // Prompt for login and password
        let login = prompt(&login_prompt, false).unwrap_or_else(|| exit(1));
        let pass = prompt("Password: ", true).unwrap_or_else(|| exit(1));
        // Check
        util::exec_wait(Duration::from_millis(1000), || {
            // Get user from prompted login
            let users_buff = fs::read_to_string(PASSWD_PATH).unwrap_or_else(|e| {
                eprintln!("login: cannot read passwd file: {e}");
                exit(1);
            });
            let user_entry = User::deserialize(&users_buff)
                .filter_map(Result::ok)
                .find(|e| e.login_name == login);
            let Some(user_entry) = user_entry else {
                return;
            };
            // Check password against user entry
            let correct = match user_entry.check_password(&pass) {
                Some(c) => c,
                // The passwd file does not have the password. Fallback onto the shadow file
                None => {
                    let shadow_buff = fs::read_to_string(SHADOW_PATH).unwrap_or_else(|e| {
                        eprintln!("login: cannot read passwd file: {e}");
                        exit(1);
                    });
                    let c = Shadow::deserialize(&shadow_buff)
                        .filter_map(Result::ok)
                        .any(|e| e.login_name == login && e.check_password(&pass));
                    c
                }
            };
            if !correct {
                return;
            }
            // Correct, setup session
            switch_user(&login, user_entry).unwrap_or_else(|e| {
                eprintln!("login: cannot initialize session: {e}");
                exit(1);
            });
        });
        eprintln!("Login incorrect");
    }
}
