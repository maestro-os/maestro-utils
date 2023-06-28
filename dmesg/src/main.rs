//! The `dmesg` command allows to print the kernel's logs.

use std::env;

/// The path to the kmsg device file.
const KMSG_PATH: &str = "/dev/kmsg";

fn main() {
    let _args: Vec<String> = env::args().collect();
    // TODO parse arguments

    // TODO read non blocking from file
    // TODO for each line:
    // - split once with `;`
    // - split left with `,`, then retrieve time, facility and level
    // - format and print
}
