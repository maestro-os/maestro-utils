//! Implements disk-related utility functions.

use super::partition::PartitionTable;
use super::util::ByteSize;
use libc::ioctl;
use std::ffi::c_long;
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::os::fd::AsRawFd;
use std::os::unix::fs::FileTypeExt;
use std::path::{Path, PathBuf};
use std::{fmt, fs, io};

/// ioctl macro: Command.
macro_rules! ioc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a) << 30) | (($b) << 8) | ($c) | (($d) << 16)
    };
}

/// ioctl macro: Read command.
#[macro_export]
macro_rules! ior {
    ($a:expr, $b:expr, $c:ty) => {
        ioc!(2, $a, $b, std::mem::size_of::<$c>() as c_long)
    };
}

/// ioctl command: Read a partitions table.
const BLKRRPART: libc::c_long = 0x125f;
/// ioctl command: Get size of disk in number of sectors.
const BLKGETSIZE64: c_long = ior!(0x12, 114, u64);

/// Returns the number of sectors on the given device.
pub fn get_disk_size(dev: &File) -> io::Result<u64> {
    let metadata = dev.metadata()?;
    let file_type = metadata.file_type();
    if file_type.is_block_device() || file_type.is_char_device() {
        let mut size = 0;
        let ret = unsafe { ioctl(dev.as_raw_fd(), BLKGETSIZE64 as _, &mut size) };
        if ret < 0 {
            return Err(Error::last_os_error());
        }
        Ok(size / 512)
    } else if file_type.is_file() {
        Ok(metadata.len() / 512)
    } else {
        Ok(0)
    }
}

/// Makes the kernel read the partition table for the given device.
///
/// If called on a non-device file, the function does nothing.
pub fn read_partitions(path: &Path) -> io::Result<()> {
    let dev = File::open(path)?;
    let ret = unsafe { ioctl(dev.as_raw_fd(), BLKRRPART as _, 0) };
    if ret < 0 {
        let err = Error::last_os_error();
        match err.raw_os_error() {
            // Inappropriate ioctl for device
            // This error is ignored because ioctl will fail when called for non-device files
            Some(25) => {}
            _ => return Err(err),
        }
    }
    Ok(())
}

/// A disk, containing partitions.
pub struct Disk {
    /// The path to the disk's device file.
    dev_path: PathBuf,
    /// The size of the disk in number of sectors.
    size: u64,
    /// The open device.
    dev: File,

    /// The partition table.
    pub partition_table: PartitionTable,
}

impl Disk {
    /// Tells whether the device file at the given path is a valid disk.
    ///
    /// This function is meant to be used when listing disks.
    fn is_valid(path: &Path) -> bool {
        let Some(path_str) = path.as_os_str().to_str() else {
            return false;
        };

        if path_str.starts_with("/dev/sd") && !path_str.contains(|c: char| c.is_numeric()) {
            return true;
        }
        if path_str.starts_with("/dev/hd") && !path_str.contains(|c: char| c.is_numeric()) {
            return true;
        }
        if path_str.starts_with("/dev/nvme0n") && !path_str.contains('p') {
            // FIXME
            return true;
        }

        // TODO Add floppy, cdrom, etc...

        false
    }

    /// Reads a disk's information from the given device path `dev_path`.
    ///
    /// If the path doesn't point to a valid device, the function returns None.
    pub fn read(dev_path: PathBuf) -> io::Result<Option<Self>> {
        let mut dev = OpenOptions::new().read(true).write(true).open(&dev_path)?;
        let Ok(size) = get_disk_size(&dev) else {
            return Ok(None);
        };
        let partition_table = PartitionTable::read(&mut dev, size)?;
        Ok(Some(Self {
            dev_path,
            size,
            dev,

            partition_table,
        }))
    }

    /// Writes the partition table to the disk.
    pub fn write(&mut self) -> io::Result<()> {
        self.partition_table.write(&mut self.dev, self.size)
    }

    /// Lists disks present on the system.
    pub fn list() -> io::Result<Vec<PathBuf>> {
        fs::read_dir("/dev")?
            .filter_map(|dev| match dev {
                Ok(dev) => {
                    let dev_path = dev.path();
                    Self::is_valid(&dev_path).then_some(Ok(dev_path))
                }
                Err(e) => Some(Err(e)),
            })
            .collect()
    }

    /// Returns the path to the device file of the disk.
    pub fn get_path(&self) -> &Path {
        &self.dev_path
    }

    /// Returns the size of the disk in number of sectors.
    pub fn get_size(&self) -> u64 {
        self.size
    }
}

impl fmt::Display for Disk {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sector_size = 512; // TODO check if this value can be different
        let byte_size = self.size * sector_size;
        writeln!(
            fmt,
            "Disk {}: {}, {byte_size} bytes, {} sectors",
            self.dev_path.display(),
            ByteSize(byte_size),
            self.size
        )?;
        writeln!(fmt, "Disk model: TODO")?; // TODO
        writeln!(
            fmt,
            "Units: sectors of 1 * {sector_size} = {sector_size} bytes",
        )?;
        writeln!(
            fmt,
            "Sector size (logical/physical): {sector_size} bytes / {sector_size} bytes"
        )?;
        writeln!(
            fmt,
            "I/O size (minimum/optimal): {sector_size} bytes / {sector_size} bytes",
        )?;
        writeln!(fmt, "Disklabel type: {}", self.partition_table.table_type)?;
        writeln!(fmt, "Disk identifier: TODO")?; // TODO
        if !self.partition_table.partitions.is_empty() {
            writeln!(fmt, "\nDevice\tStart\tEnd\tSectors\tSize\tType")?;
        }
        for p in &self.partition_table.partitions {
            writeln!(
                fmt,
                "/dev/TODO\t{}\t{}\t{}\t{}\tTODO", // TODO
                p.start,
                p.start + p.size,
                p.size,
                ByteSize(p.size)
            )?;
        }
        Ok(())
    }
}
