//! The `lsmod` command allows to list loaded kernel modules.

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::process::exit;

/// The path to the modules file.
const MODULES_PATH: &str = "/proc/modules";

fn main() {
    let file = match File::open(MODULES_PATH) {
        Ok(f) => f,

        Err(e) => {
            eprintln!("lsmod: cannot open `{}`: {}", MODULES_PATH, e);
            exit(1);
        }
    };

    let reader = BufReader::new(file);

    println!("Name\tSize\tUsed by");

    for line in reader.lines() {
        // TODO handle error
        let mut split = line.as_ref().unwrap().split(' ');

        let name = split.next().unwrap();
        let size = split.next().unwrap();
        let use_count = split.next().unwrap();
        let used_by_list = split.next().unwrap();

        // TODO padding
        println!("{}\t{}\t{}\t{}", name, size, use_count, used_by_list);
    }
}
