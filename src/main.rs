//! Main of all commands that **do not require** the SUID flag.

#![feature(option_get_or_insert_default)]
#![feature(os_str_display)]

mod dmesg;
mod fdisk;
mod insmod;
mod lsmod;
mod mkfs;
mod mount;
mod nologin;
mod powerctl;
mod ps;
mod rmmod;
mod umount;

use utils::{args, error};

fn main() {
    let (bin, args) = args();
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
        bin @ ("halt" | "poweroff" | "reboot" | "shutdown" | "suspend") => {
            powerctl::main(bin, args)
        }
        "ps" => ps::main(),
        bin @ ("useradd" | "usermod" | "userdel" | "groupadd" | "groupmod" | "groupdel") => todo!(),
        _ => error("mutils", "invalid binary name"),
    }
}
