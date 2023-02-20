//! Module handling the `ext2` filesystem.

use crate::FSFactory;
use std::cmp::max;
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io;
use std::mem::size_of;
use std::mem;
use std::num::NonZeroU32;
use std::path::Path;
use std::slice;
use utils::util::ceil_division;
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

impl Superblock {
	/// Returns the size of a block.
	pub fn get_block_size(&self) -> u32 {
		utils::util::pow2(self.block_size_log + 10) as _
	}

	/// Returns the size of an inode.
	pub fn get_inode_size(&self) -> usize {
		if self.major_version >= 1 {
			self.inode_size as _
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
	block_usage_bitmap_addr: u32,
	/// The block address of the inode usage bitmap.
	inode_usage_bitmap_addr: u32,
	/// Starting block address of inode table.
	inode_table_start_addr: u32,
	/// Number of unallocated blocks in group.
	unallocated_blocks_number: u16,
	/// Number of unallocated inodes in group.
	unallocated_inodes_number: u16,
	/// Number of directories in group.
	directories_number: u16,

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
		let mut bgd: BlockGroupDescriptor = unsafe {
			mem::zeroed()
		};
		let slice = unsafe {
			slice::from_raw_parts_mut(&mut bgd as *mut _ as *mut u8, size_of::<Self>())
		};
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
		let slice = unsafe {
			slice::from_raw_parts(self as *const _ as *const u8, size_of::<Self>())
		};
		dev.seek(SeekFrom::Start(bgd_off))?;
		dev.write_all(slice)?;

		Ok(())
	}
}

/// An inode represents a file in the filesystem. The name of the file is not
/// included in the inode but in the directory entry associated with it since
/// several entries can refer to the same inode (hard links).
#[repr(C, packed)]
struct INode {
	/// Type and permissions.
	mode: u16,
	/// User ID.
	uid: u16,
	/// Lower 32 bits of size in bytes.
	size_low: u32,
	/// Timestamp of the last modification of the metadata.
	ctime: u32,
	/// Timestamp of the last modification of the content.
	mtime: u32,
	/// Timestamp of the last access.
	atime: u32,
	/// Timestamp of the deletion.
	dtime: u32,
	/// Group ID.
	gid: u16,
	/// The number of hard links to this inode.
	hard_links_count: u16,
	/// The number of sectors used by this inode.
	used_sectors: u32,
	/// INode flags.
	flags: u32,
	/// OS-specific value.
	os_specific_0: u32,
	/// Direct block pointers.
	direct_block_ptrs: [u32; 12],
	/// Simply indirect block pointer.
	singly_indirect_block_ptr: u32,
	/// Doubly indirect block pointer.
	doubly_indirect_block_ptr: u32,
	/// Triply indirect block pointer.
	triply_indirect_block_ptr: u32,
	/// Generation number.
	generation: u32,
	/// The file's ACL.
	extended_attributes_block: u32,
	/// Higher 32 bits of size in bytes.
	size_high: u32,
	/// Block address of fragment.
	fragment_addr: u32,
	/// OS-specific value.
	os_specific_1: [u8; 12],
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
		let blk_grp = (i - 1) / superblock.inodes_per_group;
		// The offset of the inode in the block group's bitfield
		let inode_grp_off = (i - 1) % superblock.inodes_per_group;
		// The offset of the inode's block
		let inode_table_blk_off = (inode_grp_off as u64 * inode_size) / blk_size;
		// The offset of the inode in the block
		let inode_blk_off = ((i - 1) as u64 * inode_size) % blk_size;

		let bgd = BlockGroupDescriptor::read(blk_grp, superblock, dev)?;

		let a = bgd.inode_usage_bitmap_addr;

		// The block containing the inode
		let blk = bgd.inode_table_start_addr as u64 + inode_table_blk_off;

		// The offset of the inode on the disk
		Ok((blk * blk_size) + inode_blk_off)
	}
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
	let remaining_bits = end % 8;
	let aligned = remaining_bits == 0;

	for i in 0..set_bytes {
		slice[i] = 0xff;
	}

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

	/// The path the filesystem was last mounted to.
	last_mount_path: Option<String>,
}

impl FSFactory for Ext2Factory {
	fn is_present(&self, _: &Path, dev: &mut File) -> io::Result<bool> {
		let mut superblock: Superblock = unsafe {
			mem::zeroed()
		};
		let slice = unsafe {
			slice::from_raw_parts_mut(
				&mut superblock as *mut _ as *mut u8,
				size_of::<Superblock>()
			)
		};

		dev.seek(SeekFrom::Start(SUPERBLOCK_OFFSET))?;
		dev.read_exact(slice)?;

		Ok(superblock.signature == EXT2_SIGNATURE)
	}

	fn create(&self, path: &Path, dev: &mut File) -> io::Result<()> {
		let timestamp = get_timestamp().as_secs() as u32;

		let sector_size = 512; // TODO get from device
		let len = match self.len {
			Some(len) => len,
			None => utils::disk::get_disk_size(path)? * sector_size,
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
			last_fsck_timestamp: timestamp,
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

		let bgdt_off = (SUPERBLOCK_OFFSET / block_size) + 1;
		let bgdt_size = ceil_division(
			groups_count as u64 * size_of::<BlockGroupDescriptor>() as u64,
			block_size
		);
		let bgdt_end = bgdt_off + bgdt_size as u64;

		let block_usage_bitmap_size = ceil_division(
			blocks_per_group,
			(block_size * 8) as _
		);
		let inode_usage_bitmap_size = ceil_division(
			inodes_per_group,
			(block_size * 8) as _
		);
		let inodes_table_size = ceil_division(
			inodes_per_group * superblock.inode_size as u32,
			block_size as u32
		);
		let metadata_size = block_usage_bitmap_size + inode_usage_bitmap_size + inodes_table_size;

		let used_blocks_end = bgdt_end as u32 + groups_count * metadata_size;

		// Write block groups
		for i in 0..groups_count {
			let metadata_off = bgdt_end as u32 + i * metadata_size;

			let block_usage_bitmap_addr = metadata_off;
			let inode_usage_bitmap_addr = metadata_off + block_usage_bitmap_size;
			let inode_table_start_addr = metadata_off + block_usage_bitmap_size
				+ inode_usage_bitmap_size;

			let mut bgd = BlockGroupDescriptor {
				block_usage_bitmap_addr,
				inode_usage_bitmap_addr,
				inode_table_start_addr,
				unallocated_blocks_number: blocks_per_group as _,
				unallocated_inodes_number: inodes_per_group as _,
				directories_number: 0,

				_padding: [0; 14],
			};

			// Fill blocks bitmap
			let begin_block = i * blocks_per_group;
			let used_blocks_count = if begin_block < used_blocks_end {
				min(blocks_per_group, used_blocks_end - begin_block)
			} else {
				0
			};
			fill_bitmap(
				bgd.block_usage_bitmap_addr as u64 * block_size,
				block_usage_bitmap_size as usize * block_size as usize,
				used_blocks_count as usize,
				dev
			)?;
			bgd.unallocated_blocks_number -= used_blocks_count as u16;

			// Fill inodes bitmap
			let begin_inode = i * inodes_per_group;
			let used_inodes_count = if begin_inode < superblock.first_non_reserved_inode {
				min(inodes_per_group, superblock.first_non_reserved_inode - begin_inode)
			} else {
				0
			};
			fill_bitmap(
				bgd.inode_usage_bitmap_addr as u64 * block_size,
				inode_usage_bitmap_size as usize * block_size as usize,
				used_inodes_count as usize,
				dev
			)?;
			bgd.unallocated_inodes_number -= used_inodes_count as u16;

			// If containing the root inode
			if (begin_inode..(begin_inode + inodes_per_group)).contains(&ROOT_INODE) {
				bgd.directories_number += 1;
			}

			bgd.write(i, &superblock, dev)?;
		}

		// Create root directory
		let root_dir = INode {
			mode: 0x4000 | 0o755,
			uid: 0,
			size_low: 0,
			ctime: timestamp,
			mtime: timestamp,
			atime: timestamp,
			dtime: 0,
			gid: 0,
			hard_links_count: 1,
			used_sectors: 0,
			flags: 0,
			os_specific_0: 0,
			direct_block_ptrs: [0; 12],
			singly_indirect_block_ptr: 0,
			doubly_indirect_block_ptr: 0,
			triply_indirect_block_ptr: 0,
			generation: 0,
			extended_attributes_block: 0,
			size_high: 0,
			fragment_addr: 0,
			os_specific_1: [0; 12],
		};
		let root_inode_off = INode::get_disk_offset(
			NonZeroU32::new(ROOT_INODE).unwrap(),
			&superblock,
			dev
		)?;
		dev.seek(SeekFrom::Start(root_inode_off))?;
		dev.write_all(reinterpret(&root_dir))?;

		// TODO Inode for `/lost+found`
		// TODO Add entries `.`, `..` and `lost+found`

		// Write superblock
		dev.seek(SeekFrom::Start(SUPERBLOCK_OFFSET))?;
		dev.write_all(reinterpret(&superblock))?;

		Ok(())
	}
}
