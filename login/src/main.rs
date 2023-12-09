//! `login` prompts a username/password to authenticate on a new session.

#![feature(never_type)]

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::process::exit;
use std::ptr::null;
use std::time::Duration;
use std::{env, io, iter};
use utils::prompt::prompt;
use utils::user;
use utils::user::User;
use utils::util;

/// Builds an environment variable in the form: name=value
fn build_env_var(name: &str, value: impl IntoIterator<Item = u8>) -> CString {
    let data: Vec<u8> = name
        .as_bytes()
        .into_iter()
        .cloned()
        .chain(iter::once(b'='))
        .chain(value)
        .collect();
    // TODO handle when the value contains a nul-byte?
    CString::new(data).unwrap()
}

/// Switches to the given user after login is successful.
///
/// Arguments:
/// - `logname` is the name of the user used to login.
/// - `user` is the user to switch to.
fn switch_user(logname: &str, user: &User) -> io::Result<!> {
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
    let shell = if !interpreter.is_empty() {
        interpreter
    } else {
        "/bin/sh"
    };
    let path = match uid {
        0 => "/usr/local/sbin:/usr/local/bin:/sbin:/bin:/usr/sbin:/usr/bin",
        _ => "/usr/local/bin:/bin:/usr/bin",
    };
    let mail = "/var/spool/mail/".bytes().chain(login_name.bytes());

    // Build variables
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

    let bin = CString::new(shell).unwrap(); // TODO handle error?
    let argv = [bin.as_ptr(), null()];

    // Set current user
    user::set(*uid, *gid)?;
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

fn main() {
    let _args = env::args(); // TODO Parse and use

    loop {
        println!();

        // Get user prompt
        let user_prompt = format!("{} login: ", util::get_hostname());

        // Prompt login and password
        let login = prompt(Some(&user_prompt), false).unwrap_or_else(|| exit(1));
        let pass = prompt(None, true).unwrap_or_else(|| exit(1));

        // Read users lists
        let passwd = user::read_passwd(Path::new(user::PASSWD_PATH)).unwrap_or_else(|e| {
            eprintln!("Cannot read passwd file: {e}");
            exit(1);
        });
        let shadow = user::read_shadow(&Path::new(user::SHADOW_PATH)).ok();

        // Get user from prompted login
        let user_entry = passwd.into_iter().find(|e| e.login_name == login);

        let interval = Duration::from_millis(1000);
        util::exec_wait(interval, || {
            if let Some(user_entry) = user_entry {
                // Checking password against user entry
                let correct = user_entry.check_password(&pass).unwrap_or_else(|| {
                    if let Some(shadow) = shadow {
                        shadow
                            .into_iter()
                            .filter(|e| e.login_name == login)
                            .map(|e| e.check_password(&pass))
                            .next()
                            .unwrap_or(false)
                    } else {
                        false
                    }
                });

                if correct {
                    switch_user(&login, &user_entry).unwrap_or_else(|e| {
                        eprintln!("login: {e}");
                        exit(1);
                    });
                }
            }
        });

        eprintln!("Login incorrect");
    }
}
