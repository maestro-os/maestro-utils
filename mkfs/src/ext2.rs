//! Module handling the `ext2` filesystem.

use crate::FSFactory;
use std::cmp::min;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io;
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

/// The ext2 superblock structure.
#[repr(C, packed)]
struct Superblock {
	/// Total number of inodes in the filesystem.
	total_inodes: u32,
	/// Total number of blocks in the filesystem.
	total_blocks: u32,
	/// Number of blocks reserved for the superuser.
	superuser_blocks: u32,
	/// Total number of unallocated blocks.
	total_unallocated_blocks: u32,
	/// Total number of unallocated inodes.
	total_unallocated_inodes: u32,
	/// Block number of the block containing the superblock.
	superblock_block_number: u32,
	/// log2(block_size) - 10
	block_size_log: u32,
	/// log2(fragment_size) - 10
	fragment_size_log: u32,
	/// The number of blocks per block group.
	blocks_per_group: u32,
	/// The number of fragments per block group.
	fragments_per_group: u32,
	/// The number of inodes per block group.
	inodes_per_group: u32,
	/// The timestamp of the last mount operation.
	last_mount_timestamp: u32,
	/// The timestamp of the last write operation.
	last_write_timestamp: u32,
	/// The number of mounts since the last consistency check.
	mount_count_since_fsck: u16,
	/// The number of mounts allowed before a consistency check must be done.
	mount_count_before_fsck: u16,
	/// The ext2 signature.
	signature: u16,
	/// The filesystem's state.
	fs_state: u16,
	/// The action to perform when an error is detected.
	error_action: u16,
	/// The minor version.
	minor_version: u16,
	/// The timestamp of the last consistency check.
	last_fsck_timestamp: u32,
	/// The interval between mandatory consistency checks.
	fsck_interval: u32,
	/// The id os the operating system from which the filesystem was created.
	os_id: u32,
	/// The major version.
	major_version: u32,
	/// The UID of the user that can use reserved blocks.
	uid_reserved: u16,
	/// The GID of the group that can use reserved blocks.
	gid_reserved: u16,

	// Extended superblock fields

	/// The first non reserved inode
	first_non_reserved_inode: u32,
	/// The size of the inode structure in bytes.
	inode_size: u16,
	/// The block group containing the superblock.
	superblock_group: u16,
	/// Optional features for the implementation to support.
	optional_features: u32,
	/// Required features for the implementation to support.
	required_features: u32,
	/// Required features for the implementation to support for writing.
	write_required_features: u32,
	/// The filesystem id.
	filesystem_id: [u8; 16],
	/// The volume name.
	volume_name: [u8; 16],
	/// The path the volume was last mounted to.
	last_mount_path: [u8; 64],
	/// Used compression algorithms.
	compression_algorithms: u32,
	/// The number of blocks to preallocate for files.
	files_preallocate_count: u8,
	/// The number of blocks to preallocate for directories.
	directories_preallocate_count: u8,
	/// Unused.
	_unused: u16,
	/// The journal ID.
	journal_id: [u8; 16],
	/// The journal inode.
	journal_inode: u32,
	/// The journal device.
	journal_device: u32,
	/// The head of orphan inodes list.
	orphan_inode_head: u32,

	/// Structure padding.
	_padding: [u8; 788],
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

	/// The path the filesystem was last mounted to.
	last_mount_path: Option<String>,
}

impl FSFactory for Ext2Factory {
	fn is_present(&self, _dev: &mut File) -> io::Result<bool> {
		// TODO
		todo!();
	}

	fn create(&self, dev: &mut File) -> io::Result<()> {
		let len = match self.len {
			None => {
				// TODO get from device file
				todo!();
			}

			Some(len) => len,
		};

		let block_size = self.block_size.unwrap_or(DEFAULT_BLOCK_SIZE);
		// TODO if block size is not a power of two or if log2(block size) < 10, error
		let block_size_log = log2(block_size).unwrap() as u32;

		let total_blocks = (len / block_size) as u32;

		let inodes_per_group = self.inodes_per_group.unwrap_or(DEFAULT_INODES_PER_GROUP);
		let blocks_per_group = self.blocks_per_group.unwrap_or(DEFAULT_BLOCKS_PER_GROUP);

		let groups_count = total_blocks / blocks_per_group;

		let total_inodes = inodes_per_group * groups_count;

		let superblock_group = SUPERBLOCK_OFFSET as u32 / block_size as u32 / blocks_per_group;

		let volume_name = self.label
			.as_ref()
			.map(|label| {
				let label = label.as_bytes();
				let mut b: [u8; 16] = [0; 16];

				let len = min(label.len(), b.len());
				b[0..len].copy_from_slice(&label[0..len]);

				b
			})
			.unwrap_or([0; 16]);
		let last_mount_path = self.last_mount_path
			.as_ref()
			.map(|path| {
				let path = path.as_bytes();
				let mut b: [u8; 64] = [0; 64];

				let len = min(path.len(), b.len());
				b[0..len].copy_from_slice(&path[0..len]);

				b
			})
			.unwrap_or([0; 64]);
		let filesystem_id = self.fs_id.unwrap_or([0; 16]); // TODO if not set, random

		let superblock = Superblock {
			total_inodes,
			total_blocks,
			superuser_blocks: 0,
			total_unallocated_blocks: 0,
			total_unallocated_inodes: 0,
			superblock_block_number: (SUPERBLOCK_OFFSET / block_size) as _,
			block_size_log: block_size_log - 10,
			fragment_size_log: 0,
			blocks_per_group,
			fragments_per_group: 0,
			inodes_per_group,
			last_mount_timestamp: 0,
			last_write_timestamp: 0,
			mount_count_since_fsck: 0,
			mount_count_before_fsck: DEFAULT_FSCK_MOUNT_COUNT, // TODO take from param
			signature: EXT2_SIGNATURE,
			fs_state: FS_STATE_CLEAN,
			error_action: ERR_ACTION_READ_ONLY,
			minor_version: 1,
			last_fsck_timestamp: get_timestamp().as_secs() as _,
			fsck_interval: DEFAULT_FSCK_INTERVAL, // TODO take from param
			os_id: 0,
			major_version: 1,
			uid_reserved: 0,
			gid_reserved: 0,

			first_non_reserved_inode: 11,
			inode_size: 128,
			superblock_group: superblock_group as _,
			optional_features: 0,
			required_features: 0,
			write_required_features: 0,
			filesystem_id,
			volume_name,
			last_mount_path,
			compression_algorithms: 0,
			files_preallocate_count: 0,
			directories_preallocate_count: 0,
			_unused: 0,
			journal_id: [0; 16],
			journal_inode: 0,
			journal_device: 0,
			orphan_inode_head: 0,

			_padding: [0; 788],
		};

		// TODO

		dev.seek(SeekFrom::Start(SUPERBLOCK_OFFSET))?;
		dev.write(reinterpret(&superblock))?;

		Ok(())
	}
}
