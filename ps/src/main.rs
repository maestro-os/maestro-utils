//! The `ps` command allows to print the list of processes running on the system.

mod format;
mod process;
mod util;

use format::DisplayFormat;
use format::FormatParser;
use process::Process;
use process::ProcessIterator;
use std::process::exit;
use std::{env, fs, io};
use utils::user::{get_egid, get_euid, Group, User, PASSWD_PATH};

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
    // Results
    let mut selectors = Vec::new();
    let mut format = DisplayFormat::new();
    let mut default_format = true;

    // Read users and groups lists
    let users_buff = fs::read_to_string(PASSWD_PATH)?;
    let users: Vec<_> = User::deserialize(&users_buff)
        .filter_map(Result::ok)
        .collect();
    let groups_buff = fs::read_to_string(PASSWD_PATH)?;
    let groups: Vec<_> = Group::deserialize(&groups_buff)
        .filter_map(Result::ok)
        .collect();

    // TODO -l and -f
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-a" => selectors.push(Selector::Terminal),
            "-A" | "-e" => selectors.push(Selector::All),
            "-d" => selectors.push(Selector::NoLeaders),

            "-o" => {
                if let Some(format_str) = args.next() {
                    let parser = FormatParser::new(&format_str);

                    match parser.yield_format() {
                        Ok(f) => {
                            format.concat(f);
                            default_format = false;
                        }

                        Err(_) => error("invalid format"),
                    }
                } else {
                    error("format specification must follow -o");
                }
            }

            "-p" => {
                if let Some(pids_str) = args.next() {
                    let iter = match util::parse_nbr_list(&pids_str) {
                        Ok(list) => list.into_iter(),

                        Err(_) => error("process ID list syntax error"),
                    };

                    let mut pids = iter.map(Selector::Pid).collect();
                    selectors.append(&mut pids);
                } else {
                    error("list of process IDs must follow -p");
                }
            }

            "-t" => {
                if let Some(termlist) = args.next() {
                    let mut terms = util::parse_str_list(&termlist)
                        .into_iter()
                        .map(Selector::Term)
                        .collect();

                    selectors.append(&mut terms);
                } else {
                }
            }

            "-u" => {
                if let Some(users_list) = args.next() {
                    util::parse_str_list(&users_list)
                        .into_iter()
                        .for_each(|user| match users.iter().find(|u| u.login_name == user) {
                            Some(user) => selectors.push(Selector::Uid(user.uid)),

                            None => match user.parse::<u32>() {
                                Ok(uid) => selectors.push(Selector::Uid(uid)),
                                Err(_) => {}
                            },
                        });
                } else {
                    selectors.push(Selector::Uid(get_euid()));
                }
            }

            "-U" => {
                if let Some(users_list) = args.next() {
                    util::parse_str_list(&users_list)
                        .into_iter()
                        .for_each(|user| match users.iter().find(|u| u.login_name == user) {
                            Some(user) => selectors.push(Selector::Ruid(user.uid)),

                            None => match user.parse::<u32>() {
                                Ok(uid) => selectors.push(Selector::Ruid(uid)),
                                Err(_) => {}
                            },
                        });
                } else {
                    error("list of real users must follow -U");
                }
            }

            "-g" => {
                if let Some(groups_list) = args.next() {
                    util::parse_str_list(&groups_list)
                        .into_iter()
                        .for_each(
                            |group| match groups.iter().find(|g| g.group_name == group) {
                                Some(group) => selectors.push(Selector::Gid(group.gid)),

                                None => match group.parse::<u32>() {
                                    Ok(gid) => selectors.push(Selector::Gid(gid)),
                                    Err(_) => {}
                                },
                            },
                        );
                } else {
                    selectors.push(Selector::Gid(get_egid()));
                }
            }

            "-G" => {
                if let Some(groups_list) = args.next() {
                    util::parse_str_list(&groups_list)
                        .into_iter()
                        .for_each(
                            |group| match groups.iter().find(|g| g.group_name == group) {
                                Some(group) => selectors.push(Selector::Rgid(group.gid)),

                                None => match group.parse::<u32>() {
                                    Ok(gid) => selectors.push(Selector::Rgid(gid)),
                                    Err(_) => {}
                                },
                            },
                        );
                } else {
                    error("list of real groups must follow -G");
                }
            }

            _ => error("error: garbage option"),
        }
    }

    // If no selector is specified, use defaults
    if selectors.is_empty() {
        // TODO Select only processes that share the same controlling terminal
        selectors.push(Selector::Uid(get_euid()));
    }

    // If no format is specified, use default
    if default_format {
        format = DisplayFormat::default();
    }

    Ok((selectors, format))
}

fn main() {
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
