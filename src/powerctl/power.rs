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

//! This module handles power management system calls.

use std::os::raw::c_long;

/// The ID of the `reboot` system call.
const REBOOT_ID: c_long = 0x58;

/// First magic number.
const MAGIC: u32 = 0xde145e83;
/// Second magic number.
const MAGIC2: u32 = 0x40367d6e;

/// Command to power off the system.
const CMD_POWEROFF: u32 = 0;
/// Command to reboot the system.
const CMD_REBOOT: u32 = 1;
/// Command to halt the system.
const CMD_HALT: u32 = 2;
/// Command to suspend the system.
const CMD_SUSPEND: u32 = 3;

/// Power off the system.
pub fn poweroff() {
    unsafe {
        libc::syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_POWEROFF);
    }
}

/// Reboots the system.
pub fn reboot() {
    unsafe {
        libc::syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_REBOOT);
    }
}

/// Halts the system.
pub fn halt() {
    unsafe {
        libc::syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_HALT);
    }
}

/// Suspends the system.
pub fn suspend() {
    unsafe {
        libc::syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_SUSPEND);
    }
}
