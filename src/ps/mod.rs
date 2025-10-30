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

//! The `ps` command allows to print the list of processes running on the system.

mod format;
mod process;

use format::DisplayFormat;
use format::parse_display_format;
use process::Process;
use process::ProcessIterator;
use std::process::exit;
use std::{env, fs, io};
use utils::user::{Group, PASSWD_PATH, User, get_egid, get_euid};

// TODO Implement every arguments
// TODO Implement environment variables
// TODO i18n

/// Enumeration of selectors used to accept or reject a process in the list.
enum Selector {
    /// Selects processes attached to a terminal (`-a`).
    Terminal,
    /// Selects all processes (`-A`, `-e`).
    All,
    /// Selects all processes except session leaders (`-d`).
    NoLeaders,
    /// Selects all processes whose session leader effective group ID corresponds (`-g`).
    Gid(u32),
    /// Selects all processes whose real group ID corresponds (`-G`).
    Rgid(u32),
    /// Selects processes whose PID corresponds (`-p`).
    Pid(u32),
    /// Selects processes attached to the given TTY (`-t`).
    Term(String),
    /// Selects processes whose effective user ID corresponds (`-u`).
    Uid(u32),
    /// Selects processes whose real user ID corresponds (`-U`).
    Ruid(u32),
}

impl Selector {
    /// Tells whether the given process is accepted by the selector.
    pub fn is_accepted(&self, proc: &Process) -> bool {
        match self {
            Self::Terminal => proc.tty.is_some(),
            Self::All => true,
            Self::NoLeaders => {
                // TODO
                true
            }
            Self::Gid(gid) => proc.gid == *gid,
            Self::Rgid(rgid) => proc.rgid == *rgid,
            Self::Pid(pid) => proc.pid == *pid,
            Self::Term(tty) => {
                if let Some(t) = &proc.tty {
                    t == tty || t == &format!("tty{}", tty)
                } else {
                    false
                }
            }
            Self::Uid(uid) => proc.uid == *uid,
            Self::Ruid(ruid) => proc.ruid == *ruid,
        }
    }
}

/// Prints the command line usage on the standard error output.
fn print_usage() {
    eprintln!();
    eprintln!(
        "Usage: ps [-aA] [-defl] [-g grouplist] [-G grouplist] [ -n namelist] \
[-o format]... [-p proclist] [-t termlist] [ -u userlist] [-U userlist]"
    );
    eprintln!();
    eprintln!("For more details see ps(1).");
}

/// Prints an error, then exits.
fn error(msg: &str) -> ! {
    eprintln!("error: {msg}");
    print_usage();
    exit(1);
}

// TODO When a PID, UID, GID... is specified, use files' metadata to avoid reading
/// Parses arguments and returns the selectors list and format.
fn parse_args() -> io::Result<(Vec<Selector>, DisplayFormat)> {
    // Read users and groups lists
    let users_buff = fs::read_to_string(PASSWD_PATH)?;
    let users: Vec<_> = User::deserialize(&users_buff)
        .filter_map(Result::ok)
        .collect();
    let groups_buff = fs::read_to_string(PASSWD_PATH)?;
    let groups: Vec<_> = Group::deserialize(&groups_buff)
        .filter_map(Result::ok)
        .collect();
    // Results
    let mut selectors = Vec::new();
    let mut format: Option<DisplayFormat> = None;
    // TODO -l and -f
    let mut args = env::args();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-a" => selectors.push(Selector::Terminal),
            "-A" | "-e" => selectors.push(Selector::All),
            "-d" => selectors.push(Selector::NoLeaders),
            "-o" => {
                let Some(fmt) = args.next() else {
                    error("format specification must follow -o");
                };
                let Ok(mut fmt) = parse_display_format(&fmt) else {
                    error("invalid format");
                };
                let format = format.get_or_insert_default();
                format.0.append(&mut fmt.0);
            }
            "-p" => {
                let Some(pids_str) = args.next() else {
                    error("list of process IDs must follow -p");
                };
                let pids: Result<Vec<_>, _> = pids_str
                    .split(|c: char| c.is_ascii_whitespace())
                    .map(|s| s.parse().map(Selector::Pid))
                    .collect();
                let Ok(mut pids) = pids else {
                    error("process ID list syntax error");
                };
                selectors.append(&mut pids);
            }
            "-t" => {
                if let Some(termlist) = args.next() {
                    let terms = termlist
                        .split(|c: char| c.is_ascii_whitespace())
                        .map(String::from)
                        .map(Selector::Term);
                    selectors.extend(terms);
                } else {
                    // TODO
                }
            }
            "-u" => {
                if let Some(users_list) = args.next() {
                    users_list
                        .split(|c: char| c.is_ascii_whitespace())
                        .for_each(|login| {
                            let user = users.iter().find(|u| u.login_name == login);
                            match user {
                                Some(user) => selectors.push(Selector::Uid(user.uid)),
                                None => {
                                    if let Ok(uid) = login.parse::<u32>() {
                                        selectors.push(Selector::Uid(uid));
                                    }
                                }
                            }
                        });
                } else {
                    selectors.push(Selector::Uid(get_euid()));
                }
            }
            "-U" => {
                let Some(users_list) = args.next() else {
                    error("list of real users must follow -U");
                };
                users_list
                    .split(|c: char| c.is_ascii_whitespace())
                    .for_each(|user| match users.iter().find(|u| u.login_name == user) {
                        Some(user) => selectors.push(Selector::Ruid(user.uid)),
                        None => {
                            if let Ok(uid) = user.parse::<u32>() {
                                selectors.push(Selector::Ruid(uid));
                            }
                        }
                    });
            }
            "-g" => {
                if let Some(groups_list) = args.next() {
                    groups_list
                        .split(|c: char| c.is_ascii_whitespace())
                        .for_each(
                            |group| match groups.iter().find(|g| g.group_name == group) {
                                Some(group) => selectors.push(Selector::Gid(group.gid)),
                                None => {
                                    if let Ok(gid) = group.parse::<u32>() {
                                        selectors.push(Selector::Gid(gid));
                                    }
                                }
                            },
                        );
                } else {
                    selectors.push(Selector::Gid(get_egid()));
                }
            }
            "-G" => {
                let Some(groups_list) = args.next() else {
                    error("list of real groups must follow -G");
                };
                groups_list
                    .split(|c: char| c.is_ascii_whitespace())
                    .for_each(
                        |group| match groups.iter().find(|g| g.group_name == group) {
                            Some(group) => selectors.push(Selector::Rgid(group.gid)),
                            None => {
                                if let Ok(gid) = group.parse::<u32>() {
                                    selectors.push(Selector::Rgid(gid));
                                }
                            }
                        },
                    );
            }
            _ => error("error: garbage option"),
        }
    }
    // If no selector is specified, use defaults
    if selectors.is_empty() {
        // TODO Select only processes that share the same controlling terminal
        selectors.push(Selector::Uid(get_euid()));
    }
    Ok((selectors, format.unwrap_or_default()))
}

pub fn main() {
    let (selectors, format) = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("error: cannot parse arguments: {e}");
            exit(1);
        }
    };
    // Create the process iterator
    let proc_iter = match ProcessIterator::new() {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: cannot read processes list: {e}");
            exit(1);
        }
    };
    // Print header
    if format.can_print() {
        println!("{format}");
    }
    // Print processes
    proc_iter
        .filter(|proc| {
            // A process is accepted if it matches at least one selector (union)
            selectors.iter().any(|s| s.is_accepted(proc))
        })
        .for_each(|proc| println!("{}", proc.display(&format)));
}
