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

//! This module implements prompting.

use libc::ECHO;
use libc::ECHOE;
use libc::ICANON;
use libc::STDIN_FILENO;
use libc::TCSANOW;
use libc::VMIN;
use libc::tcgetattr;
use libc::tcsetattr;
use libc::termios;
use std::io::BufRead;
use std::io::Write;
use std::mem::MaybeUninit;
use std::{fmt, io};

// TODO Add line edition
/// Show a prompt. This function returns when a newline is received.
///
/// Arguments:
/// - `prompt` is the prompt's text. If `None`, the function uses the default text.
/// - `hidden` tells whether the input is hidden.
pub fn prompt<P: fmt::Display>(prompt: P, hidden: bool) -> Option<String> {
    // Save termios state
    let saved_termios = unsafe {
        let mut t: termios = MaybeUninit::zeroed().assume_init();
        tcgetattr(STDIN_FILENO, &mut t);
        t
    };
    if hidden {
        // Set temporary termios
        let mut termios = saved_termios;
        termios.c_lflag &= !(ICANON | ECHO | ECHOE);
        termios.c_cc[VMIN] = 1;
        unsafe {
            tcsetattr(STDIN_FILENO, TCSANOW, &termios);
        }
    }
    // Show prompt
    print!("{prompt}");
    let _ = io::stdout().flush();
    // Read input
    let input = io::stdin().lock().lines().next()?.unwrap_or(String::new());
    if hidden {
        println!();
        // Restore termios state
        unsafe {
            tcsetattr(STDIN_FILENO, TCSANOW, &saved_termios);
        }
    }
    Some(input)
}
