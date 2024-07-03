//! Main of all commands that **do not require** the SUID flag.

#![feature(iter_array_chunks)]
#![feature(option_get_or_insert_default)]
#![feature(os_str_display)]

mod dmesg;
mod fdisk;
mod insmod;
mod lsmod;
mod mkfs;
mod mount;
mod nologin;
mod ps;
mod rmmod;
mod umount;

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
        "fdisk" => fdisk::main(false, args),
        "sfdisk" => fdisk::main(true, args),
        "insmod" => insmod::main(args),
        "lsmod" => lsmod::main(),
        "rmmod" => rmmod::main(args),
        bin @ ("mkfs" | "mkfs.ext2") => {
            // TODO change default fs to `ext4` when implemented
            let fs_name = bin.find('.').map(|i| &bin[(i + 1)..]).unwrap_or("ext2");
            mkfs::main(fs_name, args);
        }
        "mount" => mount::main(args),
        "umount" => umount::main(args),
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
