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
