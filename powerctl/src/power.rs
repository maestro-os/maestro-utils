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

extern "C" {
    fn syscall(number: c_long, ...);
}

/// Power off the system.
pub fn poweroff() {
    unsafe {
        syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_POWEROFF);
    }
}

/// Reboots the system.
pub fn reboot() {
    unsafe {
        syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_REBOOT);
    }
}

/// Halts the system.
pub fn halt() {
    unsafe {
        syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_HALT);
    }
}

/// Suspends the system.
pub fn suspend() {
    unsafe {
        syscall(REBOOT_ID, MAGIC, MAGIC2, CMD_SUSPEND);
    }
}
