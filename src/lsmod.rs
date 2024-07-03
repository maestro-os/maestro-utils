//! The `lsmod` command allows to list loaded kernel modules.

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use utils::error;

/// The path to the modules file.
const MODULES_PATH: &str = "/proc/modules";

pub fn main() {
    let file = File::open(MODULES_PATH).unwrap_or_else(|e| {
        error("lsmod", format_args!("cannot open `{MODULES_PATH}`: {e}"));
    });
    let reader = BufReader::new(file);
    println!("Name\tSize\tUsed by");
    for line in reader.lines() {
        let line = line.unwrap_or_else(|e| {
            error("lsmod", e);
        });
        let mut split = line.split(' ');
        let name = split.next().unwrap();
        let size = split.next().unwrap();
        let use_count = split.next().unwrap();
        let used_by_list = split.next().unwrap();
        println!("{name} {size}  {use_count} {used_by_list}");
    }
}
