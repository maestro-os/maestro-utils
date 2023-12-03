//! Module handling the `ext2` filesystem.

use crate::FSFactory;
use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::mem;
use std::mem::size_of;
use std::num::NonZeroU32;
use std::slice;
use utils::util;
use utils::util::get_timestamp;
use utils::util::log2;
use utils::util::reinterpret;

/// The offset of the superblock from the beginning of the device.
const SUPERBLOCK_OFFSET: u64 = 1024;
/// The filesystem's signature.
const EXT2_SIGNATURE: u16 = 0xef53;

/// The default block size in bytes.
const DEFAULT_BLOCK_SIZE: u64 = 4096;
/// The default number of inodes per group.
const DEFAULT_INODES_PER_GROUP: u32 = 1024;
/// The default number of blocks per group.
const DEFAULT_BLOCKS_PER_GROUP: u32 = 1024;

/// The default number of mounts before a fsck pass is required.
const DEFAULT_FSCK_MOUNT_COUNT: u16 = 1024;
/// The default interval in seconds before a fsck pass is required.
const DEFAULT_FSCK_INTERVAL: u32 = 2678400;

/// Filesystem state: the filesystem is clean
const FS_STATE_CLEAN: u16 = 1;
/// Filesystem state: the filesystem has errors
const FS_STATE_ERROR: u16 = 2;

/// Error handle action: ignore
const ERR_ACTION_IGNORE: u16 = 1;
/// Error handle action: mount as read-only
const ERR_ACTION_READ_ONLY: u16 = 2;
/// Error handle action: trigger a kernel panic
const ERR_ACTION_KERNEL_PANIC: u16 = 3;

/// Optional feature: Preallocation of a specified number of blocks for each new
/// directories
const OPTIONAL_FEATURE_DIRECTORY_PREALLOCATION: u32 = 0x1;
/// Optional feature: AFS server
const OPTIONAL_FEATURE_AFS: u32 = 0x2;
/// Optional feature: Journal
const OPTIONAL_FEATURE_JOURNAL: u32 = 0x4;
/// Optional feature: Inodes have extended attributes
const OPTIONAL_FEATURE_INODE_EXTENDED: u32 = 0x8;
/// Optional feature: Filesystem can resize itself for larger partitions
const OPTIONAL_FEATURE_RESIZE: u32 = 0x10;
/// Optional feature: Directories use hash index
const OPTIONAL_FEATURE_HASH_INDEX: u32 = 0x20;

/// Required feature: Compression
const REQUIRED_FEATURE_COMPRESSION: u32 = 0x1;
/// Required feature: Directory entries have a type field
const REQUIRED_FEATURE_DIRECTORY_TYPE: u32 = 0x2;
/// Required feature: Filesystem needs to replay its journal
const REQUIRED_FEATURE_JOURNAL_REPLAY: u32 = 0x4;
/// Required feature: Filesystem uses a journal device
const REQUIRED_FEATURE_JOURNAL_DEVIXE: u32 = 0x8;

/// Write-required feature: Sparse superblocks and group descriptor tables
const WRITE_REQUIRED_SPARSE_SUPERBLOCKS: u32 = 0x1;
/// Write-required feature: Filesystem uses a 64-bit file size
const WRITE_REQUIRED_64_BITS: u32 = 0x2;
/// Directory contents are stored in the form of a Binary Tree
const WRITE_REQUIRED_DIRECTORY_BINARY_TREE: u32 = 0x4;

/// The root inode.
const ROOT_INODE: u32 = 2;

/// The ext2 superblock structure.
#[repr(C, packed)]
struct Superblock {
    /// Total number of inodes in the filesystem.
    s_inodes_count: u32,
    /// Total number of blocks in the filesystem.
    s_blocks_count: u32,
    /// Number of blocks reserved for the superuser.
    s_r_blocks_count: u32,
    /// Total number of unallocated blocks.
    s_free_blocks_count: u32,
    /// Total number of unallocated inodes.
    s_free_inodes_count: u32,
    /// Block number of the block containing the superblock.
    s_first_data_block: u32,
    /// log2(block_size) - 10
    s_log_block_size: u32,
    /// log2(fragment_size) - 10
    s_frag_log_size: u32,
    /// The number of blocks per block group.
    s_blocks_per_group: u32,
    /// The number of fragments per block group.
    s_frags_per_group: u32,
    /// The number of inodes per block group.
    s_inodes_per_group: u32,
    /// The timestamp of the last mount operation.
    s_mtime: u32,
    /// The timestamp of the last write operation.
    s_wtime: u32,
    /// The number of mounts since the last consistency check.
    s_mnt_count: u16,
    /// The number of mounts allowed before a consistency check must be done.
    s_max_mnt_count: u16,
    /// The ext2 signature.
    s_magic: u16,
    /// The filesystem's state.
    s_state: u16,
    /// The action to perform when an error is detected.
    s_errors: u16,
    /// The minor version.
    s_minor_rev_level: u16,
    /// The timestamp of the last consistency check.
    s_lastcheck: u32,
    /// The interval between mandatory consistency checks.
    s_checkinterval: u32,
    /// The id os the operating system from which the filesystem was created.
    s_creator_os: u32,
    /// The major version.
    s_rev_level: u32,
    /// The UID of the user that can use reserved blocks.
    s_def_resuid: u16,
    /// The GID of the group that can use reserved blocks.
    s_def_resgid: u16,

    // Extended superblock fields
    /// The first non reserved inode
    s_first_ino: u32,
    /// The size of the inode structure in bytes.
    s_inode_size: u16,
    /// The block group containing the superblock.
    s_block_group_nr: u16,
    /// Optional features for the implementation to support.
    s_feature_compat: u32,
    /// Required features for the implementation to support.
    s_feature_incompat: u32,
    /// Required features for the implementation to support for writing.
    s_feature_ro_compat: u32,
    /// The filesystem UUID.
    s_uuid: [u8; 16],
    /// The volume name.
    s_volume_name: [u8; 16],
    /// The path the volume was last mounted to.
    s_last_mounted: [u8; 64],
    /// Used compression algorithms.
    s_algo_bitmap: u32,
    /// The number of blocks to preallocate for files.
    s_prealloc_blocks: u8,
    /// The number of blocks to preallocate for directories.
    s_prealloc_dir_blocks: u8,
    /// Unused.
    _unused: u16,
    /// The journal UUID.
    s_journal_uuid: [u8; 16],
    /// The journal inode.
    s_journal_inum: u32,
    /// The journal device.
    s_journal_dev: u32,
    /// The head of orphan inodes list.
    s_last_orphan: u32,

    /// Structure padding.
    _padding: [u8; 788],
}

impl Superblock {
    /// Returns the size of a block.
    pub fn get_block_size(&self) -> u64 {
        util::pow2(self.s_log_block_size + 10) as _
    }

    /// Returns the size of an inode.
    pub fn get_inode_size(&self) -> usize {
        if self.s_rev_level >= 1 {
            self.s_inode_size as _
        } else {
            128
        }
    }
}

/// Structure representing a block group descriptor to be stored into the Block Group Descriptor
/// Table (BGDT).
#[repr(C, packed)]
struct BlockGroupDescriptor {
    /// The block address of the block usage bitmap.
    bg_block_bitmap: u32,
    /// The block address of the inode usage bitmap.
    bg_inode_bitmap: u32,
    /// Starting block address of inode table.
    bg_inode_table: u32,
    /// Number of unallocated blocks in group.
    bg_free_blocks_count: u16,
    /// Number of unallocated inodes in group.
    bg_free_inodes_count: u16,
    /// Number of directories in group.
    bg_used_dirs_count: u16,
    /// Structure padding.
    _padding: [u8; 14],
}

impl BlockGroupDescriptor {
    /// Returns the offset of the `i`th block group descriptor.
    ///
    /// `superblock` is the filesystem's superblock.
    pub fn get_disk_offset(i: u32, superblock: &Superblock) -> u64 {
        let bgdt_off = (SUPERBLOCK_OFFSET / superblock.get_block_size() as u64) + 1;
        (bgdt_off * superblock.get_block_size() as u64) + (i as u64 * size_of::<Self>() as u64)
    }

    /// Reads and returns the `i`th block group descriptor.
    ///
    /// Arguments:
    /// - `superblock` is the filesystem's superblock.
    /// - `dev` is the device.
    pub fn read(i: u32, superblock: &Superblock, dev: &mut File) -> io::Result<Self> {
        let bgd_off = Self::get_disk_offset(i, superblock);
        let mut bgd: BlockGroupDescriptor = unsafe { mem::zeroed() };
        let slice =
            unsafe { slice::from_raw_parts_mut(&mut bgd as *mut _ as *mut u8, size_of::<Self>()) };
        dev.seek(SeekFrom::Start(bgd_off))?;
        dev.read_exact(slice)?;
        Ok(bgd)
    }

    /// Writes the block group descriptor table.
    ///
    /// Arguments:
    /// - `i` is the offset of the group.
    /// - `superblock` is the filesystem's superblock.
    /// - `dev` is the device.
    pub fn write(&self, i: u32, superblock: &Superblock, dev: &mut File) -> io::Result<()> {
        let bgd_off = Self::get_disk_offset(i, superblock);
        let slice = reinterpret(self);
        dev.seek(SeekFrom::Start(bgd_off))?;
        dev.write_all(slice)?;
        Ok(())
    }
}

/// An inode represents a file in the filesystem.
///
/// The name of the file is not included in the inode but in the directory entry associated with it
/// since several entries can refer to the same inode (hard links).
#[repr(C, packed)]
struct INode {
    /// Type and permissions.
    i_mode: u16,
    /// User ID.
    i_uid: u16,
    /// Lower 32 bits of size in bytes.
    i_size: u32,
    /// Timestamp of the last access.
    i_atime: u32,
    /// Timestamp of inode creation.
    i_ctime: u32,
    /// Timestamp of the last modification.
    i_mtime: u32,
    /// Timestamp of the deletion.
    i_dtime: u32,
    /// Group ID.
    i_gid: u16,
    /// The number of hard links to this inode.
    i_links_count: u16,
    /// The number of sectors used by this inode.
    i_blocks: u32,
    /// INode flags.
    i_flags: u32,
    /// OS-specific value.
    i_osd1: u32,
    /// Direct block pointers.
    i_block: [u32; 15],
    /// Generation number.
    i_generation: u32,
    /// The file's ACL.
    i_file_acl: u32,
    /// Higher 32 bits of size in bytes.
    ///
    /// The name of the variable is incoherent with its purpose.
    i_dir_acl: u32,
    /// Block address of fragment.
    i_faddr: u32,
    /// OS-specific value.
    _padding: [u8; 12],
}

impl INode {
    /// Returns the offset of the inode on the disk in bytes.
    ///
    /// Arguments:
    /// - `i` is the inode's index (starting at `1`).
    /// - `superblock` is the filesystem's superblock.
    /// - `dev` is the device.
    fn get_disk_offset(i: NonZeroU32, superblock: &Superblock, dev: &mut File) -> io::Result<u64> {
        let i = i.get();

        let blk_size = superblock.get_block_size() as u64;
        let inode_size = superblock.get_inode_size() as u64;

        // The block group the inode is located in
        let blk_grp = (i - 1) / superblock.s_inodes_per_group;
        // The offset of the inode in the block group's bitfield
        let inode_grp_off = (i - 1) % superblock.s_inodes_per_group;
        // The offset of the inode's block
        let inode_table_blk_off = (inode_grp_off as u64 * inode_size) / blk_size;
        // The offset of the inode in the block
        let inode_blk_off = ((i - 1) as u64 * inode_size) % blk_size;

        let bgd = BlockGroupDescriptor::read(blk_grp, superblock, dev)?;

        // The block containing the inode
        let blk = bgd.bg_inode_table as u64 + inode_table_blk_off;

        // The offset of the inode on the disk
        Ok((blk * blk_size) + inode_blk_off)
    }
}

/// A directory entry is a structure stored in the content of an inode of type
/// `Directory`.
///
/// Each directory entry represent a file that is the stored in the
/// directory and points to its inode.
///
/// The name of the entry is not included to prevent the structure from being usized.
#[repr(C, packed)]
pub struct DirectoryEntry {
    /// The inode associated with the entry.
    inode: u32,
    /// The total size of the entry.
    rec_len: u16,
    /// Name length least-significant bits.
    name_len: u8,
    /// Name length most-significant bits or type indicator (if enabled).
    file_type: u8,
}

/// Fills the given bitmap.
///
/// Arguments:
/// - `off` is the offset to the beginning of the bitmap.
/// - `size` is the size of the bitmap in bytes.
/// - `end` is the end of the portion to be set with 1s. The rest is set with 0s.
/// - `dev` is the device.
pub fn fill_bitmap(off: u64, size: usize, end: usize, dev: &mut File) -> io::Result<()> {
    let mut slice: Vec<u8> = vec![0; size];

    let set_bytes = end / 8;
    slice[0..set_bytes].fill(0xff);

    let remaining_bits = end % 8;
    let aligned = remaining_bits == 0;
    if !aligned {
        slice[set_bytes] = (1 << remaining_bits) - 1;
    }

    dev.seek(SeekFrom::Start(off))?;
    dev.write_all(&slice)
}

/// A factory to create an `ext2` filesystem.
#[derive(Default)]
pub struct Ext2Factory {
    /// The length of the filesystem in bytes.
    len: Option<u64>,

    /// The block size in bytes.
    block_size: Option<u64>,

    /// The number of inodes per group.
    inodes_per_group: Option<u32>,
    /// The number of blocks per group.
    blocks_per_group: Option<u32>,

    /// The ID of the filesystem.
    fs_id: Option<[u8; 16]>,
    /// The name of the filesystem.
    label: Option<String>,
}

impl FSFactory for Ext2Factory {
    fn is_present(&self, dev: &mut File) -> io::Result<bool> {
        let mut superblock: Superblock = unsafe { mem::zeroed() };
        let slice = unsafe {
            slice::from_raw_parts_mut(
                &mut superblock as *mut _ as *mut u8,
                size_of::<Superblock>(),
            )
        };
        dev.seek(SeekFrom::Start(SUPERBLOCK_OFFSET))?;
        dev.read_exact(slice)?;

        Ok(superblock.s_magic == EXT2_SIGNATURE)
    }

    fn create(&self, dev: &mut File) -> io::Result<()> {
        let create_timestamp = get_timestamp().as_secs() as u32;

        let sector_size = 512; // TODO get from device
        let len = match self.len {
            Some(len) => len,
            None => utils::disk::get_disk_size(dev)? * sector_size,
        };

        let block_size = self.block_size.unwrap_or(DEFAULT_BLOCK_SIZE);
        // TODO if block size is not a power of two or if log2(block size) < 10, error
        let block_size_log = log2(block_size).unwrap() as u32;

        let blocks_per_group = self.blocks_per_group.unwrap_or(DEFAULT_BLOCKS_PER_GROUP);
        let inodes_per_group = self.inodes_per_group.unwrap_or(DEFAULT_INODES_PER_GROUP);

        let total_blocks = (len / block_size) as u32;
        let groups_count = total_blocks / blocks_per_group;
        let total_inodes = inodes_per_group * groups_count;

        let superblock_group = SUPERBLOCK_OFFSET as u32 / block_size as u32 / blocks_per_group;

        let volume_name = self
            .label
            .as_ref()
            .map(|label| {
                let label = label.as_bytes();
                let mut b: [u8; 16] = [0; 16];
                let len = min(label.len(), b.len());
                b[..len].copy_from_slice(&label[0..len]);
                b
            })
            .unwrap_or([0; 16]);
        let filesystem_id = self.fs_id.unwrap_or_else(|| {
            // Generate a random ID
            let mut id = [0; 16];
            util::get_random(&mut id);
            id
        });

        let mut superblock = Superblock {
            s_inodes_count: total_inodes,
            s_blocks_count: total_blocks,
            s_r_blocks_count: 0,
            s_free_blocks_count: 0,
            s_free_inodes_count: 0,
            s_first_data_block: (SUPERBLOCK_OFFSET / block_size) as _,
            s_log_block_size: block_size_log - 10,
            s_frag_log_size: block_size_log - 10,
            s_blocks_per_group: blocks_per_group,
            s_frags_per_group: blocks_per_group,
            s_inodes_per_group: inodes_per_group,
            s_mtime: 0,
            s_wtime: create_timestamp,
            s_mnt_count: 0,
            s_max_mnt_count: DEFAULT_FSCK_MOUNT_COUNT, // TODO take from param
            s_magic: EXT2_SIGNATURE,
            s_state: FS_STATE_CLEAN,
            s_errors: ERR_ACTION_READ_ONLY,
            s_minor_rev_level: 1,
            s_lastcheck: create_timestamp,
            s_checkinterval: DEFAULT_FSCK_INTERVAL, // TODO take from param
            s_creator_os: 0,
            s_rev_level: 1,
            s_def_resuid: 0,
            s_def_resgid: 0,

            s_first_ino: 11,
            s_inode_size: 128,
            s_block_group_nr: superblock_group as _,
            s_feature_compat: 0,
            s_feature_incompat: 0,
            s_feature_ro_compat: 0,
            s_uuid: filesystem_id,
            s_volume_name: volume_name,
            s_last_mounted: [0; 64],
            s_algo_bitmap: 0,
            s_prealloc_blocks: 0,
            s_prealloc_dir_blocks: 0,
            _unused: 0,
            s_journal_uuid: [0; 16],
            s_journal_inum: 0,
            s_journal_dev: 0,
            s_last_orphan: 0,

            _padding: [0; 788],
        };

        let bgdt_off = (SUPERBLOCK_OFFSET / block_size) + 1;
        let bgdt_size =
            (groups_count as u64 * size_of::<BlockGroupDescriptor>() as u64).div_ceil(block_size);
        let bgdt_end = bgdt_off + bgdt_size;

        let block_usage_bitmap_size = blocks_per_group.div_ceil((block_size * 8) as _);
        let inode_usage_bitmap_size = inodes_per_group.div_ceil((block_size * 8) as _);
        let inodes_table_size =
            (inodes_per_group * superblock.s_inode_size as u32).div_ceil(block_size as u32);
        let metadata_size = block_usage_bitmap_size + inode_usage_bitmap_size + inodes_table_size;

        // Add `1` to count a block for the `.` and `..` entries of root directory
        let used_blocks_end = bgdt_end as u32 + groups_count * metadata_size + 1;

        // Write block groups
        for i in 0..groups_count {
            let bg_block_bitmap = bgdt_end as u32 + i * metadata_size;
            let bg_inode_bitmap = bg_block_bitmap + block_usage_bitmap_size;
            let bg_inode_table = bg_inode_bitmap + inode_usage_bitmap_size;
            let mut bgd = BlockGroupDescriptor {
                bg_block_bitmap,
                bg_inode_bitmap,
                bg_inode_table,
                bg_free_blocks_count: blocks_per_group as _,
                bg_free_inodes_count: inodes_per_group as _,
                bg_used_dirs_count: 0,
                _padding: [0; 14],
            };

            // Fill blocks bitmap
            let begin_block = i * blocks_per_group;
            let used_blocks_count = min(
                blocks_per_group,
                used_blocks_end.saturating_sub(begin_block),
            );
            fill_bitmap(
                bg_block_bitmap as u64 * block_size,
                block_usage_bitmap_size as usize * block_size as usize,
                used_blocks_count as usize,
                dev,
            )?;
            bgd.bg_free_blocks_count -= used_blocks_count as u16;

            // Fill inodes bitmap
            let begin_inode = i * inodes_per_group;
            let used_inodes_count = min(
                inodes_per_group,
                superblock
                    .s_first_ino
                    .saturating_sub(begin_inode),
            );
            fill_bitmap(
                bg_inode_bitmap as u64 * block_size,
                inode_usage_bitmap_size as usize * block_size as usize,
                used_inodes_count as usize,
                dev,
            )?;
            bgd.bg_free_inodes_count -= used_inodes_count as u16;

            // If containing the root inode
            if (begin_inode..(begin_inode + inodes_per_group)).contains(&ROOT_INODE) {
                bgd.bg_used_dirs_count += 1;
            }

            superblock.s_free_blocks_count += bgd.bg_free_blocks_count as u32;
            superblock.s_free_inodes_count += bgd.bg_free_inodes_count as u32;

            bgd.write(i, &superblock, dev)?;
        }

        // Ensure the block size is sufficient to fit the `.` and `..` entries of the root directory
        // This should be enforced by the size of the superblock, which is larger
        assert!(block_size >= ((size_of::<DirectoryEntry>() + 8) * 2) as u64);
        // Prepare root inode for `.` and `..` entries
        let root_size_low = (block_size & 0xffffffff) as u32;
        let root_size_high = ((block_size >> 32) & 0xffffffff) as u32;

        // Create root directory
        let root_inode_id = NonZeroU32::new(ROOT_INODE).unwrap();
        let mut root_dir = INode {
            i_mode: 0x4000 | 0o755,
            i_uid: 0,
            i_size: root_size_low,
            i_atime: create_timestamp,
            i_ctime: create_timestamp,
            i_mtime: create_timestamp,
            i_dtime: 0,
            i_gid: 0,
            i_links_count: 2, // `.` and `..` entries
            i_blocks: (block_size / 512) as _,
            i_flags: 0,
            i_osd1: 0,
            i_block: [0; 15],
            i_generation: 0,
            i_file_acl: 0,
            i_dir_acl: root_size_high,
            i_faddr: 0,
            _padding: [0; 12],
        };

        // Create `.` and `..` entries for the root directory
        let entries_block = used_blocks_end - 1;
        let entries_block_off = entries_block as u64 * block_size as u64;
        root_dir.i_block[0] = entries_block;
        dev.seek(SeekFrom::Start(entries_block_off))?;
        let self_entry = DirectoryEntry {
            inode: root_inode_id.into(),
            rec_len: (size_of::<DirectoryEntry>() + 8) as _,
            name_len: 1,
            file_type: 0, // TODO fill with type when driver is compatible
        };
        dev.write_all(reinterpret(&self_entry))?;
        dev.write_all(b".")?;
        let parent_entry = DirectoryEntry {
            inode: root_inode_id.into(),
            rec_len: (block_size - (size_of::<DirectoryEntry>() + 8) as u64) as _,
            name_len: 2,
            file_type: 0, // TODO fill with type when driver is compatible
        };
        dev.seek(SeekFrom::Start(entries_block_off + 16))?;
        dev.write_all(reinterpret(&parent_entry)).unwrap();
        dev.write_all(b"..")?;

        // Write root inode
        let root_inode_off = INode::get_disk_offset(root_inode_id, &superblock, dev)?;
        dev.seek(SeekFrom::Start(root_inode_off))?;
        dev.write_all(reinterpret(&root_dir))?;

        // Write superblock
        dev.seek(SeekFrom::Start(SUPERBLOCK_OFFSET))?;
        dev.write_all(reinterpret(&superblock))?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::path::PathBuf;
    use std::process::Command;

    fn prepare_device(size: usize) -> io::Result<(PathBuf, File)> {
        let path = "/tmp/maestro-utils-test-mkfs-ext2".into();
        let mut dev = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        let sector_size = 512;
        let buf = vec![0; sector_size];
        for _ in 0..(size / sector_size) {
            dev.write_all(&buf)?;
        }
        dev.seek(SeekFrom::Start(0))?;
        Ok((path, dev))
    }

    #[test]
    pub fn check_fs() {
        let disk_size = 64 * 1024 * 1024;
        let (dev_path, mut dev) = prepare_device(disk_size).unwrap();

        let factory = Ext2Factory::default();
        factory.create(&mut dev).unwrap();

        assert!(factory.is_present(&mut dev).unwrap());

        let status = Command::new("fsck.ext2")
            .arg("-fnv")
            .arg(dev_path)
            .status()
            .unwrap();
        assert!(status.success());
    }
}
