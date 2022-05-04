//! This module implements password prompting.

use std::io::BufRead;
use std::io;

/// Termcap flags.
pub type TCFlag = u32;
/// TODO doc
pub type CC = u8;

/// Size of the array for control characters.
const NCCS: usize = 19;

/// TODO doc
const ICANON: TCFlag = 0o000002;
/// TODO doc
const ECHO: TCFlag = 0o000010;
/// TODO doc
const ECHOE: TCFlag = 0o000020;

/// Terminal IO settings.
#[repr(C)]
#[derive(Clone)]
pub struct Termios {
	/// Input modes
	pub c_iflag: TCFlag,
	/// Output modes
	pub c_oflag: TCFlag,
	/// Control modes
	pub c_cflag: TCFlag,
	/// Local modes
	pub c_lflag: TCFlag,
	/// Special characters
	pub c_cc: [CC; NCCS],
}

extern "C" {
	/// Returns the termios state of the current TTY.
	fn get_termios() -> Termios;
	/// Sets the termios state for the current TTY.
	fn set_termios(t: &Termios);
}

/// Show a prompts for a password. This function returns when a password has been entered.
/// `prompt` is the prompt's text. If None, the function uses the default text.
pub fn prompt_password(prompt: Option<String>) -> String {
	let prompt = prompt.unwrap_or("Password: ".to_owned());

	// Saving termios state
	let saved_termios = unsafe {
		get_termios()
	};

	// Setting temporary termios
	let mut termios = saved_termios.clone();
	termios.c_iflag |= ICANON;
	termios.c_iflag &= ECHO | ECHOE;
	unsafe {
		set_termios(&termios)
	}

	// Showing prompt
	print!("{}", prompt);

	// Reading password
	let mut password = io::stdin().lock().lines().next().unwrap().unwrap_or(String::new());
	// Remove newline
	password.pop();

	// Restoring termios state
	unsafe {
		set_termios(&saved_termios)
	}

	password
}
