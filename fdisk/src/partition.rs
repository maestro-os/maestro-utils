//! TODO

use crate::crc32;
use std::cmp::max;
use std::cmp::min;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::mem::size_of;
use std::path::Path;
use std::slice;
use utils::prompt::prompt;

// TODO adapt to disks whose sector size is different than 512

/// The signature of the MBR partition table.
const MBR_SIGNATURE: u16 = 0xaa55;

/// The signature in the GPT header.
const GPT_SIGNATURE: &[u8] = b"EFI PART";
/// The polynom used in the computation of the CRC32 checksum.
const GPT_CHECKSUM_POLYNOM: u32 = 0xedb88320;

/// Translates the given LBA value `lba` into a positive LBA value.
///
/// `storage_size` is the number of blocks on the storage device.
///
/// If the LBA is out of bounds of the storage device, the function returns `None`.
fn translate_lba(lba: i64, storage_size: u64) -> Option<u64> {
    if lba < 0 {
        if (-lba as u64) <= storage_size {
            Some(storage_size - (-lba as u64))
        } else {
            None
        }
    } else {
        if (lba as u64) < storage_size {
            Some(lba as _)
        } else {
            None
        }
    }
}

/// Type representing a Globally Unique IDentifier.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C, packed)]
pub struct GUID(pub [u8; 16]);

impl TryFrom<&str> for GUID {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        if s.len() != 36 {
            return Err(());
        }
        if s.chars().any(|c| !c.is_alphanumeric() && c != '-') {
            return Err(());
        }

        let mut guid = Self([0; 16]);

        let mut iter = s.chars().filter(|c| *c != '-');
        let mut i = 0;
        while let (Some(hi), Some(lo)) = (iter.next(), iter.next()) {
            let byte = String::from_iter([hi, lo]);
            // Unwrap cannot fail since characters are checked before
            let value = u8::from_str_radix(byte.as_str(), 16).unwrap();

            // Reverse necessary parts
            let index = match i {
                0..4 => 4 - i - 1,
                4..6 => 6 - i - 1 + 4,
                6..8 => 8 - i - 1 + 6,

                _ => i,
            };

            guid.0[index] = value;
            i += 1;
        }

        Ok(guid)
    }
}

impl GUID {
    /// Generates a random GUID.
    pub fn random() -> io::Result<Self> {
        let mut rand_dev = File::open("/dev/urandom")?;

        let mut s = Self([0; 16]);
        rand_dev.read_exact(&mut s.0)?;

        Ok(s)
    }
}

impl fmt::Display for GUID {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in (0..4).rev() {
            write!(fmt, "{:02x}", self.0[i])?;
        }
        write!(fmt, "-")?;

        for i in 0..2 {
            for j in (0..2).rev() {
                write!(fmt, "{:02x}", self.0[4 + i * 2 + j])?;
            }
            write!(fmt, "-")?;
        }

        for i in 8..10 {
            write!(fmt, "{:02x}", self.0[i])?;
        }
        write!(fmt, "-")?;

        for i in 10..16 {
            write!(fmt, "{:02x}", self.0[i])?;
        }

        Ok(())
    }
}

/// Structure representing a MBR partition.
#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct MBRPartition {
    /// Partition attributes.
    attrs: u8,
    /// CHS address of partition start.
    chs_start: [u8; 3],
    /// The type of the partition.
    partition_type: u8,
    /// CHS address of partition end.
    chs_end: [u8; 3],
    /// LBA address of partition start.
    lba_start: u32,
    /// The number of sectors in the partition.
    sectors_count: u32,
}

impl MBRPartition {
    /// Tells whether the partition is active.
    pub fn is_active(&self) -> bool {
        self.attrs & (1 << 7) != 0
    }
}

/// Structure representing a MBR partition table.
#[repr(C, packed)]
pub struct MBRTable {
    /// The boot code.
    boot: [u8; 440],
    /// The disk signature (optional).
    disk_signature: u32,
    /// Zero.
    zero: u16,
    /// The list of partitions.
    partitions: [MBRPartition; 4],
    /// The partition table signature.
    signature: u16,
}

/// Structure representing a GPT entry.
#[repr(C, packed)]
struct GPTEntry {
    /// The partition type's GUID.
    partition_type: GUID,
    /// The partition's GUID.
    guid: GUID,
    /// The starting LBA.
    start: i64,
    /// The ending LBA.
    end: i64,
    /// Entry's attributes.
    attributes: u64,
    /// The partition's name.
    name: [u16; 36],
}

/// Structure representing the GPT header.
#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct GPT {
    /// The header's signature.
    signature: [u8; 8],
    /// The header's revision.
    revision: u32,
    /// The size of the header in bytes.
    hdr_size: u32,
    /// The header's checksum.
    checksum: u32,
    /// Reserved field.
    reserved: u32,
    /// The LBA of the sector containing this header.
    hdr_lba: i64,
    /// The LBA of the sector containing the alternate header.
    alternate_hdr_lba: i64,
    /// The first usable sector.
    first_usable: i64,
    /// The last usable sector.
    last_usable: i64,
    /// The disk's GUID.
    disk_guid: GUID,
    /// The LBA of the beginning of the GUID partition entries array.
    entries_start: i64,
    /// The number of entries in the table.
    entries_number: u32,
    /// The size in bytes of each entry in the array.
    entry_size: u32,
    /// Checksum of the entries array.
    entries_checksum: u32,
}

/// Enumeration of partition table types.
#[derive(Debug, Eq, PartialEq)]
pub enum PartitionTableType {
    /// Master Boot Record.
    MBR,
    /// Globally Unique Identifier Partition Table.
    GPT,
}

impl PartitionTableType {
    /// Prints known partition types.
    pub fn print_partition_types(&self) {
        match self {
            Self::MBR => {
                let types = vec![
                    (0x00, "Empty"),
                    (0x01, "FAT12"),
                    (0x02, "XENIX root"),
                    (0x03, "XENIX usr"),
                    (0x04, "FAT16 <32M"),
                    (0x05, "Extended"),
                    (0x06, "FAT16"),
                    (0x07, "HPFS/NTFS/exFAT"),
                    (0x08, "AIX"),
                    (0x09, "AIX bootable"),
                    (0x0a, "OS/2 Boot Manager"),
                    (0x0b, "W95 FAT32"),
                    (0x0c, "W95 FAT32 (LBA)"),
                    (0x0e, "W95 FAT16 (LBA)"),
                    (0x0f, "W95 Ext'd (LBA)"),
                    (0x10, "OPUS"),
                    (0x11, "Hidden FAT12"),
                    (0x12, "Compaq diagnostics"),
                    (0x14, "Hidden FAT16 <3"),
                    (0x16, "Hidden FAT16"),
                    (0x17, "Hidden HPFS/NTFS"),
                    (0x18, "AST SmartSleep"),
                    (0x1b, "Hidden W95 FAT3"),
                    (0x1c, "Hidden W95 FAT3"),
                    (0x1e, "Hidden W95 FAT1"),
                    (0x24, "NEC DOS"),
                    (0x27, "Hidden NTFS Win"),
                    (0x39, "Plan 9"),
                    (0x3c, "PartitionMagic"),
                    (0x40, "Venix 80286"),
                    (0x41, "PPC PReP Boot"),
                    (0x42, "SFS"),
                    (0x4d, "QNX4.x"),
                    (0x4e, "QNX4.x 2nd part"),
                    (0x4f, "QNX4.x 3rd part"),
                    (0x50, "OnTrack DM"),
                    (0x51, "OnTrack DM6 Aux"),
                    (0x52, "CP/M"),
                    (0x53, "OnTrack DM6 Aux"),
                    (0x54, "OnTrackDM6"),
                    (0x55, "EZ-Drive"),
                    (0x56, "Golden Bow"),
                    (0x5c, "Priam Edisk"),
                    (0x61, "SpeedStor"),
                    (0x63, "GNU HURD or Sys"),
                    (0x64, "Novell Netware"),
                    (0x65, "Novell Netware"),
                    (0x70, "DiskSecure Mult"),
                    (0x75, "PC/IX"),
                    (0x80, "Old Minix"),
                    (0x81, "Minix / old Linux"),
                    (0x82, "Linux swap / Solaris"),
                    (0x83, "Linux"),
                    (0x84, "OS/2 hidden"),
                    (0x85, "Linux extended"),
                    (0x86, "NTFS volume set"),
                    (0x87, "NTFS volume set"),
                    (0x88, "Linux plaintext"),
                    (0x8e, "Linux LVM"),
                    (0x93, "Amoeba"),
                    (0x94, "Amoeba BBT"),
                    (0x9f, "BSD/OS"),
                    (0xa0, "IBM Thinkpad"),
                    (0xa5, "FreeBSD"),
                    (0xa6, "OpenBSD"),
                    (0xa7, "NeXTSTEP"),
                    (0xa8, "Darwin UFS"),
                    (0xa9, "NetBSD"),
                    (0xab, "Darwin boot"),
                    (0xaf, "HFS / HFS+"),
                    (0xb7, "BSDI fs"),
                    (0xb8, "BSDI swap"),
                    (0xbb, "Boot Wizard hidden"),
                    (0xbc, "Acronis FAT32"),
                    (0xbe, "Solaris boot"),
                    (0xbf, "Solaris"),
                    (0xc1, "DRDOS/sec"),
                    (0xc4, "DRDOS/sec"),
                    (0xc6, "DRDOS/sec"),
                    (0xc7, "Syrinx"),
                    (0xda, "Non-FS data"),
                    (0xdb, "CP/M / CTOS / ."),
                    (0xde, "Dell Utility"),
                    (0xdf, "BootIt"),
                    (0xe0, "ST AVFS"),
                    (0xe1, "DOS access"),
                    (0xe3, "DOS R/O"),
                    (0xe4, "SpeedStor"),
                    (0xea, "Linux extended"),
                    (0xeb, "BeOS fs"),
                    (0xee, "GPT"),
                    (0xef, "EFI (FAT-12/16/32)"),
                    (0xf0, "Linux/PA-RISC bootloader"),
                    (0xf1, "SpeedStor"),
                    (0xf2, "DOS secondary"),
                    (0xf4, "SpeedStor"),
                    (0xf8, "EBBR protective"),
                    (0xfb, "VMware VMFS"),
                    (0xfc, "VMware VMKCORE"),
                    (0xfd, "Linux raid auto"),
                    (0xfe, "LANstep"),
                    (0xff, "BBT"),
                ];
                let max_len = types.iter().map(|(_, name)| name.len()).max().unwrap_or(0);
                let term_width = 80; // TODO get from ioctl
                let entries_per_line = max(term_width / (max_len + 5), 1);

                for (i, (id, name)) in types.iter().enumerate() {
                    print!("  {:02x} {:max_len$}", id, name);

                    if i % entries_per_line == entries_per_line - 1 {
                        println!();
                    }
                }
            }

            Self::GPT => {
                let types = vec![
                    ("EFI System", "c12a7328-f81f-11d2-ba4b-00a0c93ec93b"),
                    (
                        "MBR partition scheme",
                        "024dee41-33e7-11d3-9d69-0008c781f39f",
                    ),
                    ("Intel Fast Flash", "d3bfe2de-3daf-11df-ba40-e3a556d89593"),
                    ("BIOS boot", "21686148-6449-6e6f-744e-656564454649"),
                    (
                        "Sony boot partition",
                        "f4019732-066e-4e12-8273-346c5641494f",
                    ),
                    (
                        "Lenovo boot partition",
                        "bfbfafe7-a34f-448a-9a5b-6213eb736c22",
                    ),
                    ("PowerPC PReP boot", "9e1a2d38-c612-4316-aa26-8b49521e5a8b"),
                    ("ONIE boot", "7412f7d5-a156-4b13-81dc-867174929325"),
                    ("ONIE config", "d4e6e2cd-4469-46f3-b5cb-1bff57afc149"),
                    ("Microsoft reserved", "e3c9e316-0b5c-4db8-817d-f92df00215ae"),
                    (
                        "Microsoft basic data",
                        "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7",
                    ),
                    (
                        "Microsoft LDM metadata",
                        "5808c8aa-7e8f-42e0-85d2-e1e90434cfb3",
                    ),
                    ("Microsoft LDM data", "af9b60a0-1431-4f62-bc68-3311714a69ad"),
                    (
                        "Windows recovery environment",
                        "de94bba4-06d1-4d40-a16a-bfd50179d6ac",
                    ),
                    (
                        "IBM General Parallel Fs",
                        "37affc90-ef7d-4e96-91c3-2d7ae055b174",
                    ),
                    (
                        "Microsoft Storage Spaces",
                        "e75caf8f-f680-4cee-afa3-b001e56efc2d",
                    ),
                    ("HP-UX data", "75894c1e-3aeb-11d3-b7c1-7b03a0000000"),
                    ("HP-UX service", "e2a1e728-32e3-11d6-a682-7b03a0000000"),
                    ("Linux swap", "0657fd6d-a4ab-43c4-84e5-0933c84b4f4f"),
                    ("Linux filesystem", "0fc63daf-8483-4772-8e79-3d69d8477de4"),
                    ("Linux server data", "3b8f8425-20e0-4f3b-907f-1a25a76f98e8"),
                    ("Linux root (x86)", "44479540-f297-41b2-9af7-d131d5f0458a"),
                    (
                        "Linux root (x86-64)",
                        "4f68bce3-e8cd-4db1-96e7-fbcaf984b709",
                    ),
                    ("Linux root (Alpha)", "6523f8ae-3eb1-4e2a-a05a-18b695ae656f"),
                    ("Linux root (ARC)", "d27f46ed-2919-4cb8-bd25-9531f3c16534"),
                    ("Linux root (ARM)", "69dad710-2ce4-4e3c-b16c-21a1d49abed3"),
                    (
                        "Linux root (ARM-64)",
                        "b921b045-1df0-41c3-af44-4c6f280d3fae",
                    ),
                    ("Linux root (IA-64)", "993d8d3d-f80e-4225-855a-9daf8ed7ea97"),
                    (
                        "Linux root (LoongArch-64)",
                        "77055800-792c-4f94-b39a-98c91b762bb6",
                    ),
                    (
                        "Linux root (MIPS-32 LE)",
                        "37c58c8a-d913-4156-a25f-48b1b64e07f0",
                    ),
                    (
                        "Linux root (MIPS-64 LE)",
                        "700bda43-7a34-4507-b179-eeb93d7a7ca3",
                    ),
                    ("Linux root (PPC)", "1de3f1ef-fa98-47b5-8dcd-4a860a654d78"),
                    ("Linux root (PPC64)", "912ade1d-a839-4913-8964-a10eee08fbd2"),
                    (
                        "Linux root (PPC64LE)",
                        "c31c45e6-3f39-412e-80fb-4809c4980599",
                    ),
                    (
                        "Linux root (RISC-V-32)",
                        "60d5a7fe-8e7d-435c-b714-3dd8162144e1",
                    ),
                    (
                        "Linux root (RISC-V-64)",
                        "72ec70a6-cf74-40e6-bd49-4bda08e8f224",
                    ),
                    ("Linux root (S390)", "08a7acea-624c-4a20-91e8-6e0fa67d23f9"),
                    ("Linux root (S390X)", "5eead9a9-fe09-4a1e-a1d7-520d00531306"),
                    (
                        "Linux root (TILE-Gx)",
                        "c50cdd70-3862-4cc3-90e1-809a8c93ee2c",
                    ),
                    ("Linux reserved", "8da63339-0007-60c0-c436-083ac8230908"),
                    ("Linux home", "933ac7e1-2eb4-4f13-b844-0e14e2aef915"),
                    ("Linux RAID", "a19d880f-05fc-4d3b-a006-743f0f84911e"),
                    ("Linux LVM", "e6d6d379-f507-44c2-a23c-238f2a3df928"),
                    (
                        "Linux variable data",
                        "4d21b016-b534-45c2-a9fb-5c16e091fd2d",
                    ),
                    (
                        "Linux temporary data",
                        "7ec6f557-3bc5-4aca-b293-16ef5df639d1",
                    ),
                    ("Linux /usr (x86)", "75250d76-8cc6-458e-bd66-bd47cc81a812"),
                    (
                        "Linux /usr (x86-64)",
                        "8484680c-9521-48c6-9c11-b0720656f69e",
                    ),
                    ("Linux /usr (Alpha)", "e18cf08c-33ec-4c0d-8246-c6c6fb3da024"),
                    ("Linux /usr (ARC)", "7978a683-6316-4922-bbee-38bff5a2fecc"),
                    ("Linux /usr (ARM)", "7d0359a3-02b3-4f0a-865c-654403e70625"),
                    (
                        "Linux /usr (ARM-64)",
                        "b0e01050-ee5f-4390-949a-9101b17104e9",
                    ),
                    ("Linux /usr (IA-64)", "4301d2a6-4e3b-4b2a-bb94-9e0b2c4225ea"),
                    (
                        "Linux /usr (LoongArch-64)",
                        "e611c702-575c-4cbe-9a46-434fa0bf7e3f",
                    ),
                    (
                        "Linux /usr (MIPS-32 LE)",
                        "0f4868e9-9952-4706-979f-3ed3a473e947",
                    ),
                    (
                        "Linux /usr (MIPS-64 LE)",
                        "c97c1f32-ba06-40b4-9f22-236061b08aa8",
                    ),
                    ("Linux /usr (PPC)", "7d14fec5-cc71-415d-9d6c-06bf0b3c3eaf"),
                    ("Linux /usr (PPC64)", "2c9739e2-f068-46b3-9fd0-01c5a9afbcca"),
                    (
                        "Linux /usr (PPC64LE)",
                        "15bb03af-77e7-4d4a-b12b-c0d084f7491c",
                    ),
                    (
                        "Linux /usr (RISC-V-32)",
                        "b933fb22-5c3f-4f91-af90-e2bb0fa50702",
                    ),
                    (
                        "Linux /usr (RISC-V-64)",
                        "beaec34b-8442-439b-a40b-984381ed097d",
                    ),
                    ("Linux /usr (S390)", "cd0f869b-d0fb-4ca0-b141-9ea87cc78d66"),
                    ("Linux /usr (S390X)", "8a4f5770-50aa-4ed3-874a-99b710db6fea"),
                    (
                        "Linux /usr (TILE-Gx)",
                        "55497029-c7c1-44cc-aa39-815ed1558630",
                    ),
                    (
                        "Linux root verity (x86)",
                        "d13c5d3b-b5d1-422a-b29f-9454fdc89d76",
                    ),
                    (
                        "Linux root verity (x86-64)",
                        "2c7357ed-ebd2-46d9-aec1-23d437ec2bf5",
                    ),
                    (
                        "Linux root verity (Alpha)",
                        "fc56d9e9-e6e5-4c06-be32-e74407ce09a5",
                    ),
                    (
                        "Linux root verity (ARC)",
                        "24b2d975-0f97-4521-afa1-cd531e421b8d",
                    ),
                    (
                        "Linux root verity (ARM)",
                        "7386cdf2-203c-47a9-a498-f2ecce45a2d6",
                    ),
                    (
                        "Linux root verity (ARM-64)",
                        "df3300ce-d69f-4c92-978c-9bfb0f38d820",
                    ),
                    (
                        "Linux root verity (IA-64)",
                        "86ed10d5-b607-45bb-8957-d350f23d0571",
                    ),
                    (
                        "Linux root verity (LoongArch-64)",
                        "f3393b22-e9af-4613-a948-9d3bfbd0c535",
                    ),
                    (
                        "Linux root verity (MIPS-32 LE)",
                        "d7d150d2-2a04-4a33-8f12-16651205ff7b",
                    ),
                    (
                        "Linux root verity (MIPS-64 LE)",
                        "16b417f8-3e06-4f57-8dd2-9b5232f41aa6",
                    ),
                    (
                        "Linux root verity (PPC)",
                        "98cfe649-1588-46dc-b2f0-add147424925",
                    ),
                    (
                        "Linux root verity (PPC64)",
                        "9225a9a3-3c19-4d89-b4f6-eeff88f17631",
                    ),
                    (
                        "Linux root verity (PPC64LE)",
                        "906bd944-4589-4aae-a4e4-dd983917446a",
                    ),
                    (
                        "Linux root verity (RISC-V-32)",
                        "ae0253be-1167-4007-ac68-43926c14c5de",
                    ),
                    (
                        "Linux root verity (RISC-V-64)",
                        "b6ed5582-440b-4209-b8da-5ff7c419ea3d",
                    ),
                    (
                        "Linux root verity (S390)",
                        "7ac63b47-b25c-463b-8df8-b4a94e6c90e1",
                    ),
                    (
                        "Linux root verity (S390X)",
                        "b325bfbe-c7be-4ab8-8357-139e652d2f6b",
                    ),
                    (
                        "Linux root verity (TILE-Gx)",
                        "966061ec-28e4-4b2e-b4a5-1f0a825a1d84",
                    ),
                    (
                        "Linux /usr verity (x86)",
                        "8f461b0d-14ee-4e81-9aa9-049b6fb97abd",
                    ),
                    (
                        "Linux /usr verity (x86-64)",
                        "77ff5f63-e7b6-4633-acf4-1565b864c0e6",
                    ),
                    (
                        "Linux /usr verity (Alpha)",
                        "8cce0d25-c0d0-4a44-bd87-46331bf1df67",
                    ),
                    (
                        "Linux /usr verity (ARC)",
                        "fca0598c-d880-4591-8c16-4eda05c7347c",
                    ),
                    (
                        "Linux /usr verity (ARM)",
                        "c215d751-7bcd-4649-be90-6627490a4c05",
                    ),
                    (
                        "Linux /usr verity (ARM-64)",
                        "6e11a4e7-fbca-4ded-b9e9-e1a512bb664e",
                    ),
                    (
                        "Linux /usr verity (IA-64)",
                        "6a491e03-3be7-4545-8e38-83320e0ea880",
                    ),
                    (
                        "Linux /usr verity (LoongArch-64)",
                        "f46b2c26-59ae-48f0-9106-c50ed47f673d",
                    ),
                    (
                        "Linux /usr verity (MIPS-32 LE)",
                        "46b98d8d-b55c-4e8f-aab3-37fca7f80752",
                    ),
                    (
                        "Linux /usr verity (MIPS-64 LE)",
                        "3c3d61fe-b5f3-414d-bb71-8739a694a4ef",
                    ),
                    (
                        "Linux /usr verity (PPC)",
                        "df765d00-270e-49e5-bc75-f47bb2118b09",
                    ),
                    (
                        "Linux /usr verity (PPC64)",
                        "bdb528a5-a259-475f-a87d-da53fa736a07",
                    ),
                    (
                        "Linux /usr verity (PPC64LE)",
                        "ee2b9983-21e8-4153-86d9-b6901a54d1ce",
                    ),
                    (
                        "Linux /usr verity (RISC-V-32)",
                        "cb1ee4e3-8cd0-4136-a0a4-aa61a32e8730",
                    ),
                    (
                        "Linux /usr verity (RISC-V-64)",
                        "8f1056be-9b05-47c4-81d6-be53128e5b54",
                    ),
                    (
                        "Linux /usr verity (S390)",
                        "b663c618-e7bc-4d6d-90aa-11b756bb1797",
                    ),
                    (
                        "Linux /usr verity (S390X)",
                        "31741cc4-1a2a-4111-a581-e00b447d2d06",
                    ),
                    (
                        "Linux /usr verity (TILE-Gx)",
                        "2fb4bf56-07fa-42da-8132-6b139f2026ae",
                    ),
                    (
                        "Linux root verity sign. (x86)",
                        "5996fc05-109c-48de-808b-23fa0830b676",
                    ),
                    (
                        "Linux root verity sign. (x86-64)",
                        "41092b05-9fc8-4523-994f-2def0408b176",
                    ),
                    (
                        "Linux root verity sign. (Alpha)",
                        "d46495b7-a053-414f-80f7-700c99921ef8",
                    ),
                    (
                        "Linux root verity sign. (ARC)",
                        "143a70ba-cbd3-4f06-919f-6c05683a78bc",
                    ),
                    (
                        "Linux root verity sign. (ARM)",
                        "42b0455f-eb11-491d-98d3-56145ba9d037",
                    ),
                    (
                        "Linux root verity sign. (ARM-64)",
                        "6db69de6-29f4-4758-a7a5-962190f00ce3",
                    ),
                    (
                        "Linux root verity sign. (IA-64)",
                        "e98b36ee-32ba-4882-9b12-0ce14655f46a",
                    ),
                    (
                        "Linux root verity sign. (LoongArch-64)",
                        "5afb67eb-ecc8-4f85-ae8e-ac1e7c50e7d0",
                    ),
                    (
                        "Linux root verity sign. (MIPS-32 LE)",
                        "c919cc1f-4456-4eff-918c-f75e94525ca5",
                    ),
                    (
                        "Linux root verity sign. (MIPS-64 LE)",
                        "904e58ef-5c65-4a31-9c57-6af5fc7c5de7",
                    ),
                    (
                        "Linux root verity sign. (PPC)",
                        "1b31b5aa-add9-463a-b2ed-bd467fc857e7",
                    ),
                    (
                        "Linux root verity sign. (PPC64)",
                        "f5e2c20c-45b2-4ffa-bce9-2a60737e1aaf",
                    ),
                    (
                        "Linux root verity sign. (PPC64LE)",
                        "d4a236e7-e873-4c07-bf1d-bf6cf7f1c3c6",
                    ),
                    (
                        "Linux root verity sign. (RISC-V-32)",
                        "3a112a75-8729-4380-b4cf-764d79934448",
                    ),
                    (
                        "Linux root verity sign. (RISC-V-64)",
                        "efe0f087-ea8d-4469-821a-4c2a96a8386a",
                    ),
                    (
                        "Linux root verity sign. (S390)",
                        "3482388e-4254-435a-a241-766a065f9960",
                    ),
                    (
                        "Linux root verity sign. (S390X)",
                        "c80187a5-73a3-491a-901a-017c3fa953e9",
                    ),
                    (
                        "Linux root verity sign. (TILE-Gx)",
                        "b3671439-97b0-4a53-90f7-2d5a8f3ad47b",
                    ),
                    (
                        "Linux /usr verity sign. (x86)",
                        "974a71c0-de41-43c3-be5d-5c5ccd1ad2c0",
                    ),
                    (
                        "Linux /usr verity sign. (x86-64)",
                        "e7bb33fb-06cf-4e81-8273-e543b413e2e2",
                    ),
                    (
                        "Linux /usr verity sign. (Alpha)",
                        "5c6e1c76-076a-457a-a0fe-f3b4cd21ce6e",
                    ),
                    (
                        "Linux /usr verity sign. (ARC)",
                        "94f9a9a1-9971-427a-a400-50cb297f0f35",
                    ),
                    (
                        "Linux /usr verity sign. (ARM)",
                        "d7ff812f-37d1-4902-a810-d76ba57b975a",
                    ),
                    (
                        "Linux /usr verity sign. (ARM-64)",
                        "c23ce4ff-44bd-4b00-b2d4-b41b3419e02a",
                    ),
                    (
                        "Linux /usr verity sign. (IA-64)",
                        "8de58bc2-2a43-460d-b14e-a76e4a17b47f",
                    ),
                    (
                        "Linux /usr verity sign. (LoongArch-64)",
                        "b024f315-d330-444c-8461-44bbde524e99",
                    ),
                    (
                        "Linux /usr verity sign. (MIPS-32 LE)",
                        "3e23ca0b-a4bc-4b4e-8087-5ab6a26aa8a9",
                    ),
                    (
                        "Linux /usr verity sign. (MIPS-64 LE)",
                        "f2c2c7ee-adcc-4351-b5c6-ee9816b66e16",
                    ),
                    (
                        "Linux /usr verity sign. (PPC)",
                        "7007891d-d371-4a80-86a4-5cb875b9302e",
                    ),
                    (
                        "Linux /usr verity sign. (PPC64)",
                        "0b888863-d7f8-4d9e-9766-239fce4d58af",
                    ),
                    (
                        "Linux /usr verity sign. (PPC64LE)",
                        "c8bfbd1e-268e-4521-8bba-bf314c399557",
                    ),
                    (
                        "Linux /usr verity sign. (RISC-V-32)",
                        "c3836a13-3137-45ba-b583-b16c50fe5eb4",
                    ),
                    (
                        "Linux /usr verity sign. (RISC-V-64)",
                        "d2f9000a-7a18-453f-b5cd-4d32f77a7b32",
                    ),
                    (
                        "Linux /usr verity sign. (S390)",
                        "17440e4f-a8d0-467f-a46e-3912ae6ef2c5",
                    ),
                    (
                        "Linux /usr verity sign. (S390X)",
                        "3f324816-667b-46ae-86ee-9b0c0c6c11b4",
                    ),
                    (
                        "Linux /usr verity sign. (TILE-Gx)",
                        "4ede75e2-6ccc-4cc8-b9c7-70334b087510",
                    ),
                    (
                        "Linux extended boot",
                        "bc13c2ff-59e6-4262-a352-b275fd6f7172",
                    ),
                    ("Linux user's home", "773f91ef-66d4-49b5-bd83-d683bf40ad16"),
                    ("FreeBSD data", "516e7cb4-6ecf-11d6-8ff8-00022d09712b"),
                    ("FreeBSD boot", "83bd6b9d-7f41-11dc-be0b-001560b84f0f"),
                    ("FreeBSD swap", "516e7cb5-6ecf-11d6-8ff8-00022d09712b"),
                    ("FreeBSD UFS", "516e7cb6-6ecf-11d6-8ff8-00022d09712b"),
                    ("FreeBSD ZFS", "516e7cba-6ecf-11d6-8ff8-00022d09712b"),
                    ("FreeBSD Vinum", "516e7cb8-6ecf-11d6-8ff8-00022d09712b"),
                    ("Apple HFS/HFS+", "48465300-0000-11aa-aa11-00306543ecac"),
                    ("Apple APFS", "7c3457ef-0000-11aa-aa11-00306543ecac"),
                    ("Apple UFS", "55465300-0000-11aa-aa11-00306543ecac"),
                    ("Apple RAID", "52414944-0000-11aa-aa11-00306543ecac"),
                    ("Apple RAID offline", "52414944-5f4f-11aa-aa11-00306543ecac"),
                    ("Apple boot", "426f6f74-0000-11aa-aa11-00306543ecac"),
                    ("Apple label", "4c616265-6c00-11aa-aa11-00306543ecac"),
                    ("Apple TV recovery", "5265636f-7665-11aa-aa11-00306543ecac"),
                    ("Apple Core storage", "53746f72-6167-11aa-aa11-00306543ecac"),
                    ("Apple Silicon boot", "69646961-6700-11aa-aa11-00306543ecac"),
                    (
                        "Apple Silicon recovery",
                        "52637672-7900-11aa-aa11-00306543ecac",
                    ),
                    ("Solaris boot", "6a82cb45-1dd2-11b2-99a6-080020736631"),
                    ("Solaris root", "6a85cf4d-1dd2-11b2-99a6-080020736631"),
                    (
                        "Solaris /usr & Apple ZFS",
                        "6a898cc3-1dd2-11b2-99a6-080020736631",
                    ),
                    ("Solaris swap", "6a87c46f-1dd2-11b2-99a6-080020736631"),
                    ("Solaris backup", "6a8b642b-1dd2-11b2-99a6-080020736631"),
                    ("Solaris /var", "6a8ef2e9-1dd2-11b2-99a6-080020736631"),
                    ("Solaris /home", "6a90ba39-1dd2-11b2-99a6-080020736631"),
                    (
                        "Solaris alternate sector",
                        "6a9283a5-1dd2-11b2-99a6-080020736631",
                    ),
                    ("Solaris reserved 1", "6a945a3b-1dd2-11b2-99a6-080020736631"),
                    ("Solaris reserved 2", "6a9630d1-1dd2-11b2-99a6-080020736631"),
                    ("Solaris reserved 3", "6a980767-1dd2-11b2-99a6-080020736631"),
                    ("Solaris reserved 4", "6a96237f-1dd2-11b2-99a6-080020736631"),
                    ("Solaris reserved 5", "6a8d2ac7-1dd2-11b2-99a6-080020736631"),
                    ("NetBSD swap", "49f48d32-b10e-11dc-b99b-0019d1879648"),
                    ("NetBSD FFS", "49f48d5a-b10e-11dc-b99b-0019d1879648"),
                    ("NetBSD LFS", "49f48d82-b10e-11dc-b99b-0019d1879648"),
                    (
                        "NetBSD concatenated",
                        "2db519c4-b10f-11dc-b99b-0019d1879648",
                    ),
                    ("NetBSD encrypted", "2db519ec-b10f-11dc-b99b-0019d1879648"),
                    ("NetBSD RAID", "49f48daa-b10e-11dc-b99b-0019d1879648"),
                    ("ChromeOS kernel", "fe3a2a5d-4f32-41a7-b725-accc3285a309"),
                    ("ChromeOS root fs", "3cb8e202-3b7e-47dd-8a3c-7ff2a13cfcec"),
                    ("ChromeOS reserved", "2e0a753d-9e48-43b0-8337-b15192cb1b5e"),
                    ("MidnightBSD data", "85d5e45a-237c-11e1-b4b3-e89a8f7fc3a7"),
                    ("MidnightBSD boot", "85d5e45e-237c-11e1-b4b3-e89a8f7fc3a7"),
                    ("MidnightBSD swap", "85d5e45b-237c-11e1-b4b3-e89a8f7fc3a7"),
                    ("MidnightBSD UFS", "0394ef8b-237e-11e1-b4b3-e89a8f7fc3a7"),
                    ("MidnightBSD ZFS", "85d5e45d-237c-11e1-b4b3-e89a8f7fc3a7"),
                    ("MidnightBSD Vinum", "85d5e45c-237c-11e1-b4b3-e89a8f7fc3a7"),
                    ("Ceph Journal", "45b0969e-9b03-4f30-b4c6-b4b80ceff106"),
                    (
                        "Ceph Encrypted Journal",
                        "45b0969e-9b03-4f30-b4c6-5ec00ceff106",
                    ),
                    ("Ceph OSD", "4fbd7e29-9d25-41b8-afd0-062c0ceff05d"),
                    ("Ceph crypt OSD", "4fbd7e29-9d25-41b8-afd0-5ec00ceff05d"),
                    (
                        "Ceph disk in creation",
                        "89c57f98-2fe5-4dc0-89c1-f3ad0ceff2be",
                    ),
                    (
                        "Ceph crypt disk in creation",
                        "89c57f98-2fe5-4dc0-89c1-5ec00ceff2be",
                    ),
                    ("VMware VMFS", "aa31e02a-400f-11db-9590-000c2911d1b8"),
                    ("VMware Diagnostic", "9d275380-40ad-11db-bf97-000c2911d1b8"),
                    ("VMware Virtual SAN", "381cfccc-7288-11e0-92ee-000c2911d0b2"),
                    ("VMware Virsto", "77719a0c-a4a0-11e3-a47e-000c29745a24"),
                    ("VMware Reserved", "9198effc-31c0-11db-8f78-000c2911d1b8"),
                    ("OpenBSD data", "824cc7a0-36a8-11e3-890a-952519ad3f61"),
                    ("QNX6 file system", "cef5a9ad-73bc-4601-89f3-cdeeeee321a1"),
                    ("Plan 9 partition", "c91818f9-8025-47af-89d2-f030d7000c2c"),
                    ("HiFive FSBL", "5b193300-fc78-40cd-8002-e86c45580b47"),
                    ("HiFive BBL", "2e54b353-1271-4842-806f-e436d6af6985"),
                    ("Haiku BFS", "42465331-3ba3-10f1-802a-4861696b7521"),
                    (
                        "Marvell Armada 3700 Boot partition",
                        "6828311a-ba55-42a4-bcde-a89bb5edecae",
                    ),
                ];
                let max_len = types.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

                for (i, (name, uuid)) in types.iter().enumerate() {
                    print!("{:3} {:max_len$} {}", i, name, uuid);
                }
            }
        }
    }

    // TODO Return result instead
    /// Prompts for informations related to a new partition to be created.
    pub fn prompt_new_partition(&self) -> Partition {
        let (_extended, max_partition_count) = match self {
            Self::MBR => {
                // TODO get info from disk, to be passed as argument
                println!("Partition type");
                println!("   p   primary (TODO primary, TODO extended, TODO free)");
                println!("   e   extended (container for logical partitions)");

                let extended = prompt(Some("Select (default p): "), false)
                    .map(|s| s == "e") // TODO handle invalid prompt (other than `p` and `e`)
                    .unwrap_or(false);

                (extended, 4)
            }

            Self::GPT => (false, 128),
        };

        // Ask partition number
        let first = 1; // TODO get from disk
        let prompt_str = format!(
            "Partition number ({}-{}, default {}): ",
            first, max_partition_count, first
        );
        let partition_number = prompt(Some(&prompt_str), false)
            .map(|s| s.parse::<usize>())
            .transpose()
            .unwrap() // TODO handle error
            .unwrap_or(first);

        // Ask first sector
        let first_available = 2048; // TODO
        let last_available = 0; // TODO
        let prompt_str = format!(
            "First sector ({}-{}, default {})",
            first_available, last_available, first_available
        );
        let start = prompt(Some(&prompt_str), false)
            .map(|s| s.parse::<u64>())
            .transpose()
            .unwrap() // TODO handle error
            .unwrap_or(first_available);

        // Ask last sector
        let prompt_str = format!(
            "Last sector, +/-sectors or +/-size{{K,M,G,T,P}} ({}-{}, default {})",
            start, last_available, last_available
        );
        let end = prompt(Some(&prompt_str), false)
            .map(|s| {
                // TODO parse suffix
                s.parse::<u64>()
            })
            .transpose()
            .unwrap() // TODO handle error
            .unwrap_or(last_available);

        let sector_size = 512; // TODO get from disk?
        let size = (end - start) / sector_size as u64;

        // TODO use other values?
        let part_type = match self {
            Self::MBR => PartitionType::MBR(0),
            Self::GPT => PartitionType::GPT(GUID([0; 16])),
        };

        Partition {
            start,
            size,

            part_type,

            uuid: None, // TODO

            bootable: false,
        }
    }

    /// Reads partitions from the storage device represented by `dev` and returns the list.
    pub fn read(&self, dev: &mut File, sectors_count: u64) -> io::Result<Option<Vec<Partition>>> {
        match self {
            Self::MBR => {
                let mut buff: [u8; size_of::<MBRTable>()] = [0; size_of::<MBRTable>()];
                dev.seek(SeekFrom::Start(0))?;
                dev.read_exact(&mut buff)?;

                let mbr = unsafe { &*(buff.as_ptr() as *const MBRTable) };
                if mbr.signature != MBR_SIGNATURE {
                    return Ok(None);
                }

                let parts = mbr
                    .partitions
                    .iter()
                    .filter(|p| p.sectors_count > 0)
                    .map(|p| Partition {
                        start: p.lba_start as _,
                        size: p.sectors_count as _,

                        part_type: PartitionType::MBR(p.partition_type),

                        uuid: None,

                        bootable: p.is_active(),
                    })
                    .collect();
                Ok(Some(parts))
            }

            Self::GPT => {
                let mut buff: [u8; size_of::<GPT>()] = [0; size_of::<GPT>()];
                dev.seek(SeekFrom::Start(512))?;
                dev.read_exact(&mut buff)?;

                let hdr = unsafe { &mut *(buff.as_mut_ptr() as *mut GPT) };
                // Check signature
                if hdr.signature != GPT_SIGNATURE {
                    return Ok(None);
                }

                let mut crc32_table: [u32; 256] = [0; 256];
                crc32::compute_lookuptable(&mut crc32_table, GPT_CHECKSUM_POLYNOM);

                // Check header checksum
                let checksum = hdr.checksum;
                hdr.checksum = 0;
                // TODO computation must be done with the size of the header (dynamic)
                if crc32::compute(&buff, &crc32_table) != checksum {
                    // TODO invalid table
                    todo!();
                }

                // TODO check entries checksum
                // TODO if entries checksum is invalid, use alternate table

                let mut parts = Vec::new();

                let sector_size = 512; // TODO
                let entries_off =
                    translate_lba(hdr.entries_start, sector_size).unwrap() * sector_size;

                for i in 0..hdr.entries_number {
                    let off = entries_off + i as u64 * hdr.entry_size as u64;

                    let mut buff = vec![0; hdr.entry_size as usize];
                    dev.seek(SeekFrom::Start(off as _))?;
                    dev.read_exact(&mut buff)?;

                    let entry = unsafe { &*(buff.as_ptr() as *const GPTEntry) };

                    // If entry is unused, skip
                    if entry.guid.0.iter().all(|i| *i == 0) {
                        continue;
                    }

                    // TODO handle negative lba
                    parts.push(Partition {
                        start: entry.start as _,
                        size: (entry.end - entry.start) as _,

                        part_type: PartitionType::GPT(entry.partition_type),

                        uuid: Some(entry.guid),

                        bootable: false,
                    });
                }

                Ok(Some(parts))
            }
        }
    }

    /// Writes a GPT header and partitions.
    fn write_gpt(
        dev: &mut File,
        storage_size: u64,
        hdr_off: i64,
        hdr: &GPT,
        parts: &[GPTEntry],
    ) -> io::Result<()> {
        let sector_size = 512; // TODO

        let hdr_off = translate_lba(hdr_off, storage_size).unwrap() * sector_size;
        let entries_off = translate_lba(hdr.entries_start, storage_size).unwrap() * sector_size;

        for (i, entry) in parts.iter().enumerate() {
            let off = entries_off + i as u64 * size_of::<GPTEntry>() as u64;

            let entry_slice = unsafe {
                slice::from_raw_parts(entry as *const _ as *const _, size_of::<GPTEntry>())
            };
            dev.seek(SeekFrom::Start(off))?;
            dev.write_all(entry_slice)?;
        }

        let hdr_slice =
            unsafe { slice::from_raw_parts(hdr as *const _ as *const _, size_of::<GPT>()) };
        dev.seek(SeekFrom::Start(hdr_off))?;
        dev.write_all(hdr_slice)?;

        Ok(())
    }

    /// Writes the partitions table to the storage device represented by `dev`.
    ///
    /// Arguments:
    /// - `dev` is the file representing the device.
    /// - `partitions` is the list of partitions to be written.
    /// - `sectors_count` is the number of sectors on the disk.
    pub fn write(
        &self,
        dev: &mut File,
        partitions: &[Partition],
        sectors_count: u64,
    ) -> io::Result<()> {
        match self {
            Self::MBR => {
                let mut mbr = MBRTable {
                    boot: [0; 440],
                    disk_signature: 0,
                    zero: 0,
                    partitions: [MBRPartition::default(); 4],
                    signature: MBR_SIGNATURE,
                };

                if partitions.len() > mbr.partitions.len() {
                    // TODO error
                    todo!();
                }

                for (i, p) in partitions.iter().enumerate() {
                    let partition_type = match p.part_type {
                        PartitionType::MBR(t) => t,
                        _ => panic!(),
                    };

                    mbr.partitions[i] = MBRPartition {
                        attrs: 0,
                        chs_start: [0; 3],
                        partition_type,
                        chs_end: [0; 3],
                        lba_start: p.start as _,
                        sectors_count: p.size as _,
                    };
                }

                let slice = unsafe {
                    slice::from_raw_parts(
                        (&mbr as *const _ as *const u8).add(mbr.boot.len()),
                        size_of::<MBRTable>() - mbr.boot.len(),
                    )
                };
                dev.seek(SeekFrom::Start(mbr.boot.len() as _))?;
                dev.write_all(slice)
            }

            Self::GPT => {
                if partitions.len() > 128 {
                    // TODO error
                    todo!();
                }

                // Write protective MBR
                Self::MBR.write(
                    dev,
                    &[Partition {
                        start: 1,
                        size: min(u32::MAX as u64, sectors_count - 1),

                        part_type: PartitionType::MBR(0xee),

                        uuid: None,

                        bootable: true,
                    }],
                    sectors_count,
                )?;

                let disk_guid = GUID::random()?;

                // Primary table
                let mut gpt = GPT {
                    signature: [0; 8],
                    revision: 0x010000,
                    hdr_size: size_of::<GPT>() as _,
                    checksum: 0,
                    reserved: 0,
                    hdr_lba: 1,
                    alternate_hdr_lba: -1,
                    first_usable: 34,
                    last_usable: -34,
                    disk_guid,
                    entries_start: 2,
                    entries_number: partitions.len() as _,
                    entry_size: 128,
                    entries_checksum: 0,
                };
                gpt.signature.copy_from_slice(GPT_SIGNATURE);

                let parts: Vec<GPTEntry> = partitions
                    .iter()
                    .map(|p| {
                        let partition_type = match p.part_type {
                            PartitionType::GPT(i) => i,
                            _ => panic!(),
                        };

                        GPTEntry {
                            partition_type,
                            guid: p.uuid.unwrap(),
                            start: p.start as _,
                            end: (p.start + p.size) as _,
                            attributes: 0, // TODO
                            name: [0; 36], // TODO
                        }
                    })
                    .collect();

                let mut crc32_table: [u32; 256] = [0; 256];
                crc32::compute_lookuptable(&mut crc32_table, GPT_CHECKSUM_POLYNOM);

                let parts_slice = unsafe {
                    slice::from_raw_parts(
                        parts.as_ptr() as *const u8,
                        parts.len() * size_of::<GPTEntry>(),
                    )
                };
                gpt.entries_checksum = crc32::compute(parts_slice, &crc32_table);

                let hdr_slice = unsafe {
                    slice::from_raw_parts(&gpt as *const _ as *const u8, size_of::<GPT>())
                };
                gpt.checksum = crc32::compute(hdr_slice, &crc32_table);

                Self::write_gpt(dev, sectors_count, 1, &gpt, &parts)?;

                // Alternate table
                gpt.checksum = 0;
                gpt.alternate_hdr_lba = 1;
                gpt.entries_start = -33;
                let hdr_slice = unsafe {
                    slice::from_raw_parts(&gpt as *const _ as *const u8, size_of::<GPT>())
                };
                gpt.checksum = crc32::compute(hdr_slice, &crc32_table);
                Self::write_gpt(dev, sectors_count, -1, &gpt, &parts)?;

                Ok(())
            }
        }
    }
}

impl fmt::Display for PartitionTableType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MBR => write!(fmt, "dos"),
            Self::GPT => write!(fmt, "gpt"),
        }
    }
}

/// Enumeration of partition type formats.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PartitionType {
    /// MBR partition type.
    MBR(u8),
    /// GPT partition type.
    GPT(GUID),
}

impl Default for PartitionType {
    fn default() -> Self {
        Self::MBR(0)
    }
}

impl TryFrom<&str> for PartitionType {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        GUID::try_from(s)
            .map(Self::GPT)
            .or_else(|_| u8::from_str_radix(s, 16).map(Self::MBR))
            .map_err(|_| ())
    }
}

impl fmt::Display for PartitionType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MBR(n) => write!(fmt, "{:x}", n),
            Self::GPT(n) => write!(fmt, "{}", n),
        }
    }
}

/// Structure storing informations about a partition.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Partition {
    /// The start offset in sectors.
    pub start: u64,
    /// The size of the partition in sectors.
    pub size: u64,

    /// The partition type.
    pub part_type: PartitionType,

    /// The partition's UUID.
    pub uuid: Option<GUID>,

    /// Tells whether the partition is bootable.
    pub bootable: bool,
}

impl fmt::Display for Partition {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "start={}, size={}, type={}",
            self.start, self.size, self.part_type
        )?;

        if self.bootable {
            write!(fmt, ", bootable")?;
        }

        if let Some(ref uuid) = self.uuid {
            write!(fmt, ", uuid={}", uuid)?;
        }

        Ok(())
    }
}

/// Structure representing a partition table.
#[derive(Debug, Eq, PartialEq)]
pub struct PartitionTable {
    /// The type of the partition table.
    pub table_type: PartitionTableType,
    /// The list of partitions in the table.
    pub partitions: Vec<Partition>,
}

impl PartitionTable {
    /// Reads the partition table from the given device file.
    ///
    /// Arguments:
    /// - `dev` is the device to read from.
    /// - `sectors_count` is the number of sectors on the device.
    ///
    /// The cursor of the device might be changed by the function.
    ///
    /// If the table is invalid, the function returns an empty MBR table.
    pub fn read(dev: &mut File, sectors_count: u64) -> io::Result<Self> {
        for t in [PartitionTableType::GPT, PartitionTableType::MBR] {
            if let Some(partitions) = t.read(dev, sectors_count)? {
                return Ok(PartitionTable {
                    table_type: t,
                    partitions,
                });
            }
        }
        Ok(PartitionTable {
            table_type: PartitionTableType::MBR,
            partitions: vec![],
        })
    }

    /// Writes the partition table to the disk device.
    ///
    /// Arguments:
    /// - `dev` is the device to write on.
    /// - `sectors_count` is the number of sectors on the device.
    pub fn write(&self, dev: &mut File, sectors_count: u64) -> io::Result<()> {
        self.table_type.write(dev, &self.partitions, sectors_count)
    }

    /// Serializes a partitions list into a sfdisk script.
    ///
    /// `dev` is the path to the device file of the disk.
    ///
    /// The function returns the resulting script.
    pub fn serialize(&self, dev: &Path) -> String {
        let mut script = String::new();

        // Writing header
        // TODO label
        // TODO label-id
        script += format!("device: {}\n", dev.display()).as_str();
        script += "unit: sectors\n";
        script += "\n";

        // Writing partitions
        for (i, p) in self.partitions.iter().enumerate() {
            script += &format!("{}{} : {}\n", dev.display(), i, p);
        }

        script
    }

    /// Deserializes a partitions list from a given sfdisk script.
    ///
    /// The function returns the list of partitions.
    pub fn deserialize(script: &str) -> Result<Self, String> {
        // Skip header
        let mut iter = script.split('\n');
        for line in iter.by_ref() {
            if line.trim().is_empty() {
                break;
            }
        }

        // Parse partitions
        let mut partitions = vec![];
        for line in iter {
            if line.trim().is_empty() {
                continue;
            }

            let mut split = line.split(':').skip(1);
            let Some(values) = split.next() else {
                return Err("Invalid syntax".to_owned());
            };

            // Filling partition structure
            let mut part = Partition::default();
            for v in values.split(',') {
                let mut split = v.split('=');
                let Some(name) = split.next() else {
                    return Err("Invalid syntax".to_owned());
                };

                let name = name.trim();
                let value = split.next().map(|s| s.trim());

                match name {
                    "start" => {
                        let Some(val) = value else {
                            return Err("`start` requires a value".into());
                        };
                        let Ok(v) = val.parse() else {
                            return Err(format!("Invalid value for `start`: {}", val));
                        };

                        part.start = v;
                    }

                    "size" => {
                        let Some(val) = value else {
                            return Err("`size` requires a value".into());
                        };
                        let Ok(v) = val.parse() else {
                            return Err(format!("Invalid value for `size`: {}", val));
                        };

                        part.size = v;
                    }

                    "type" => {
                        let Some(val) = value else {
                            return Err("`type` requires a value".into());
                        };
                        let Ok(v) = val.try_into() else {
                            return Err(format!("Invalid value for `type`: {}", val));
                        };

                        part.part_type = v;
                    }

                    "uuid" => {
                        let Some(val) = value else {
                            return Err("`uuid` requires a value".into());
                        };
                        let Ok(val) = val.try_into() else {
                            return Err(format!("Invalid value for `uuid`: {}", val));
                        };

                        part.uuid = Some(val);
                    }

                    "bootable" => part.bootable = true,

                    _ => return Err(format!("Unknown attribute: `{}`", name)),
                }
            }

            partitions.push(part);
        }

        Ok(Self {
            table_type: PartitionTableType::MBR, // TODO
            partitions,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn partitions_serialize0() {
        let table0 = PartitionTable {
            table_type: PartitionTableType::MBR,
            partitions: vec![],
        };

        let script = table0.serialize(&PathBuf::from("/dev/sda"));
        let table1 = PartitionTable::deserialize(&script).unwrap();

        assert_eq!(table0, table1);
    }

    #[test]
    fn partitions_serialize1() {
        let table0 = PartitionTable {
            table_type: PartitionTableType::MBR,
            partitions: vec![Partition {
                start: 0,
                size: 1,

                part_type: PartitionType::MBR(0xab),

                uuid: Some(GUID([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])),

                bootable: false,
            }],
        };

        let script = table0.serialize(&PathBuf::from("/dev/sda"));
        let table1 = PartitionTable::deserialize(&script).unwrap();

        assert_eq!(table0, table1);
    }

    // TODO More tests (especially invalid scripts)
}
