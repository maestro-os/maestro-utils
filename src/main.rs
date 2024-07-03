//! Main of all commands that **do not require** the SUID flag.

#![feature(option_get_or_insert_default)]
#![feature(os_str_display)]

mod dmesg;
mod insmod;
mod lsmod;
mod mount;
mod nologin;
mod ps;
mod rmmod;

use std::env;
use utils::error;

fn main() {
    let mut args = env::args_os();
    let bin = args
        .next()
        .and_then(|s| s.into_string().ok())
        .unwrap_or_else(|| {
            error("mutils", "missing binary name");
        });
    match bin.as_str() {
        "dmesg" => dmesg::main(),
        "fdisk" => todo!(),
        "insmod" => insmod::main(args),
        "lsmod" => lsmod::main(),
        "rmmod" => rmmod::main(args),
        bin @ ("mkfs" | "mkfs.ext2") => {
            // TODO change default fs to `ext4` when implemented
            let fs_name = bin.find(".").map(|i| &bin[(i + 1)..]).unwrap_or("ext2");
            todo!()
        }
        "mount" => mount::main(args),
        "umount" => todo!(),
        "nologin" => nologin::main(),
        "powerctl" => todo!(),
        "halt" => todo!(),
        "poweroff" => todo!(),
        "reboot" => todo!(),
        "shutdown" => todo!(),
        "suspend" => todo!(),
        "ps" => ps::main(),
        "useradd" => todo!(),
        "usermod" => todo!(),
        "userdel" => todo!(),
        "groupadd" => todo!(),
        "groupmod" => todo!(),
        "groupdel" => todo!(),
        _ => error("mutils", "invalid binary name"),
    }
}
