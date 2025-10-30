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

//! The `dmesg` command allows to print the kernel's logs.

/// The path to the kmsg device file.
const KMSG_PATH: &str = "/dev/kmsg";

pub fn main() {
    // TODO read non blocking from file
    // TODO for each line:
    // - split once with `;`
    // - split left with `,`, then retrieve time, facility and level
    // - format and print
}
