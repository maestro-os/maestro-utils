//! The `ps` command allows to print the list of processes running on the system.

mod process;

use process::Process;
use std::env;

/// Enumeration of selectors used to accept or reject a process in the list.
enum Selector {
	/// Selects processes attached to a terminal (-a).
	Terminal,
	/// Selects all processes (-A, -e).
	All,
	/// Selects all processes except session leaders (-d).
	NoLeaders,
	/// Selects all processes whose session leader group ID corresponds (-g).
	Gid(u32),
	/// Selects all processes whose real group ID corresponds (-G).
	Rgid(u32),
	/// Selects processes whose PID corresponds (-p).
	Pid(u32),
	/// Selects processes attached to the given TTY (-t).
	Term(String),
	/// Selects processes whose user ID corresponds (-u).
	Uid(u32),
	/// Selects processes whose real user ID corresponds (-U).
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
				todo!();
			},

			Self::Gid(gid) => proc.gid == *gid,
			Self::Rgid(rgid) => proc.rgid == *rgid,
			Self::Pid(pid) => proc.pid == *pid,

			Self::Term(tty) => if let Some(t) = &proc.tty {
				t == tty || t == &format!("tty{}", tty)
			} else {
				false
			},

			Self::Uid(uid) => proc.uid == *uid,
			Self::Ruid(ruid) => proc.ruid == *ruid,
		}
	}
}

fn main() {
	let _args: Vec<String> = env::args().collect(); // TODO Parse and use

	let _selectors = Vec::<Selector>::new();
	// TODO Fill
	// TODO If no filter is specified, use default

	// TODO Print column description

	// Creating the process iterator and filtering processing according to arguments
	// A process is accepted if it matches at least one selector (union)
	let proc_iter = ProcessIterator::new().filter(| proc | {
		for s in selectors {
			if s.is_accepted(proc) {
				return true;
			}
		}

		false
	});

	for _proc in proc_iter {
		// TODO Print with format
	}
}
