//! The CP/M file system on an Altair 8-inch disk image, enough for
//! `altair put` to copy files onto it.
//!
//! The layout comes from Mike Douglas's CP/M 2.2b BIOS, [BIOS.ASM]. A track
//! holds 32 sectors of 137 bytes, and each sector carries one CP/M record
//! of 128 bytes:
//!
//! ```text
//! track 0-1    the boot loader and CP/M itself, not files
//! track 2-76   the file system: 2400 records in 150 blocks of 2 KB
//!                block 0-1     the directory, 64 entries of 32 bytes
//!                block 2-149   the files
//! ```
//!
//! CP/M counts records from track 2 up. Two tables turn a record into a
//! sector on the disk:
//!
//! ```text
//! record 37 ─► track 2 + 37 / 32 = 3, sector 37 % 32 = 5
//!           ─► the BIOS sector: TRANSLATE[5] = 10
//!           ─► the Altair skew, on track 6 and up: odd sectors swap halves
//!           ─► position (3 * 32 + 10) * 137 in the image
//! ```
//!
//! The BIOS spreads the sectors over the track, so that CP/M has time to
//! handle one record before the next one passes under the head.
//!
//! Tracks 0 to 5 keep the system format, and tracks 6 to 76 the data
//! format. Both wrap the record in a header and a stop byte:
//!
//! ```text
//! system: │ track │ 00 01 │ 128 data bytes │ FF │ sum │ 00 ...
//! data:   │ track │ sector │ 00 00 │ sum │ 00 00 │ 128 data bytes │ FF │ 00
//! ```
//!
//! The track byte has bit 7 set, and the sum is the low byte of the sum of
//! the data bytes. The BIOS reads a sector with a wrong sum as bad.
//!
//! A directory entry names a file and lists its blocks. One entry, an
//! extent, holds up to 16 KB: a bigger file takes an entry for each 16 KB.
//!
//! [BIOS.ASM]: https://deramp.com/downloads/altair/software/8_inch_floppy/CPM/CPM%202.2/CPM%202.2B/BIOS.ASM

use altair::cards::DISK_BYTES;

/// The bytes of a sector on the disk.
const SECTOR_BYTES: usize = 137;
/// The sectors on a track.
const SECTORS: usize = 32;
/// The bytes of a CP/M record, the data in a sector.
const RECORD_BYTES: usize = 128;
/// The tracks with the boot loader and CP/M, below the file system.
const RESERVED_TRACKS: usize = 2;
/// The first track in the data format. The tracks below it keep the system
/// format.
const DATA_FORMAT_TRACK: usize = 6;
/// The records in an allocation block of 2 KB.
const BLOCK_RECORDS: usize = 16;
/// The allocation blocks of the file system, the directory blocks among
/// them.
const BLOCKS: usize = 150;
/// The blocks at the start that hold the directory.
const DIRECTORY_BLOCKS: usize = 2;
/// The entries in the directory.
const DIRECTORY_ENTRIES: usize = 64;
/// The bytes of a directory entry.
const ENTRY_BYTES: usize = 32;
/// The records that one directory entry holds: 16 KB.
const EXTENT_RECORDS: usize = 128;
/// The blocks that one directory entry holds.
const EXTENT_BLOCKS: usize = EXTENT_RECORDS / BLOCK_RECORDS;
/// The extents that the extent byte of an entry counts before the next
/// extent byte, S2, goes up.
const EXTENTS_PER_S2: usize = 32;
/// The user byte of a free directory entry.
const FREE_ENTRY: u8 = 0xE5;
/// The user area that `put` writes to: user 0, the one `A>` starts in.
const USER: u8 = 0x00;
/// The bytes of a file name, eight for the name and three for the type.
const NAME_BYTES: usize = 11;
/// The characters that CP/M doesn't allow in a file name.
const FORBIDDEN_CHARACTERS: &str = "<>.,;:=?*[]";
/// The byte that fills the rest of the last record: Ctrl+Z, the end of a
/// CP/M text file.
const END_OF_FILE_BYTE: u8 = 0x1A;
/// The bit that the BIOS sets in the track byte of each sector.
const SYNC_BIT: u8 = 0x80;
/// The byte after the data of each sector.
const STOP_BYTE: u8 = 0xFF;
/// The word 0100H that the BIOS puts after the track byte of a sector in
/// the system format, low byte first.
const SYSTEM_WORD: [u8; 2] = [0x00, 0x01];
/// The BIOS sector for each CP/M sector of a track: the BIOS table
/// `tranTbl`, counted from 0.
const TRANSLATE: [u8; SECTORS] = [
    0, 8, 16, 24, 2, 10, 18, 26, 4, 12, 20, 28, 6, 14, 22, 30, //
    1, 9, 17, 25, 3, 11, 19, 27, 5, 13, 21, 29, 7, 15, 23, 31,
];
/// The bit that the Altair skew flips in an odd sector on a track in the
/// data format.
const SKEW_BIT: u8 = 0x10;

/// Where a sector of each format keeps its parts.
mod offset {
    /// The track byte, in both formats.
    pub const TRACK: usize = 0;
    /// The word 0100H in the system format.
    pub const SYSTEM_WORD: usize = 1;
    /// The data in the system format.
    pub const SYSTEM_DATA: usize = 3;
    /// The stop byte in the system format.
    pub const SYSTEM_STOP: usize = 131;
    /// The sum in the system format.
    pub const SYSTEM_SUM: usize = 132;
    /// The BIOS sector in the data format.
    pub const DATA_SECTOR: usize = 1;
    /// The sum in the data format.
    pub const DATA_SUM: usize = 4;
    /// The data in the data format.
    pub const DATA_DATA: usize = 7;
    /// The stop byte in the data format.
    pub const DATA_STOP: usize = 135;
}

/// Where in a directory entry CP/M keeps each field.
mod entry {
    /// The user area, or [`FREE_ENTRY`](super::FREE_ENTRY).
    pub const USER: usize = 0;
    /// The name and the type, padded with spaces.
    pub const NAME: usize = 1;
    /// The extent number, the low five bits.
    pub const EXTENT: usize = 12;
    /// The extent number, the bits above the low five.
    pub const S2: usize = 14;
    /// The records in this extent.
    pub const RECORDS: usize = 15;
    /// The blocks of this extent, one byte each.
    pub const BLOCKS: usize = 16;
}

/// An Altair 8-inch disk image with CP/M 2.2 on it, held in memory.
pub struct CpmDisk {
    bytes: Vec<u8>,
}

impl CpmDisk {
    /// Takes the bytes of a disk image. Fails when they don't fill an
    /// 8-inch disk. An image can run past the disk: the one on deramp.com
    /// pads it to a multiple of 256 bytes.
    pub fn new(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() < DISK_BYTES {
            return Err(format!(
                "not an 8-inch Altair disk: it holds {} bytes, short of {DISK_BYTES}",
                bytes.len()
            ));
        }
        Ok(Self { bytes })
    }

    /// Returns the bytes of the image.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Puts `data` on the disk as file `name` in user 0, in place of a
    /// file with that name. Leaves the disk as it was when the file
    /// doesn't fit.
    pub fn put(&mut self, name: &str, data: &[u8]) -> Result<(), String> {
        let name = file_name(name)?;
        let records = data.len().div_ceil(RECORD_BYTES);
        let extents = records.div_ceil(EXTENT_RECORDS).max(1);
        let needed_blocks = records.div_ceil(BLOCK_RECORDS);

        let old_entries = self.entries_of(&name);
        let free_entries = self.free_entries(&old_entries);
        let free_blocks = self.free_blocks(&old_entries);
        if free_entries.len() < extents {
            return Err("the directory is full".into());
        }
        if free_blocks.len() < needed_blocks {
            return Err(format!(
                "the disk is full: the file needs {} KB, and {} KB are free",
                needed_blocks * 2,
                free_blocks.len() * 2
            ));
        }

        for index in old_entries {
            let mut free = self.entry(index);
            free[entry::USER] = FREE_ENTRY;
            self.write_entry(index, &free);
        }
        for (index, chunk) in data.chunks(RECORD_BYTES).enumerate() {
            let mut record = [END_OF_FILE_BYTE; RECORD_BYTES];
            record[..chunk.len()].copy_from_slice(chunk);
            let block = free_blocks[index / BLOCK_RECORDS];
            self.write_record(block * BLOCK_RECORDS + index % BLOCK_RECORDS, &record);
        }
        let blocks = &free_blocks[..needed_blocks];
        for (extent, &index) in free_entries.iter().take(extents).enumerate() {
            let mut new = [0; ENTRY_BYTES];
            new[entry::USER] = USER;
            new[entry::NAME..entry::NAME + NAME_BYTES].copy_from_slice(&name);
            new[entry::EXTENT] = (extent % EXTENTS_PER_S2) as u8;
            new[entry::S2] = (extent / EXTENTS_PER_S2) as u8;
            new[entry::RECORDS] = (records - extent * EXTENT_RECORDS).min(EXTENT_RECORDS) as u8;
            let extent_blocks = blocks.chunks(EXTENT_BLOCKS).nth(extent).unwrap_or(&[]);
            for (slot, &block) in extent_blocks.iter().enumerate() {
                new[entry::BLOCKS + slot] = block as u8;
            }
            self.write_entry(index, &new);
        }
        Ok(())
    }

    /// Returns the directory entries of file `name` in user 0.
    fn entries_of(&self, name: &[u8; NAME_BYTES]) -> Vec<usize> {
        (0..DIRECTORY_ENTRIES)
            .filter(|&index| {
                let entry = self.entry(index);
                // Bit 7 of the type bytes marks a file read-only or a system
                // file, and bit 7 of a name byte can carry other flags.
                entry[entry::USER] == USER
                    && entry[entry::NAME..entry::NAME + NAME_BYTES]
                        .iter()
                        .zip(name)
                        .all(|(byte, letter)| byte & 0x7F == *letter)
            })
            .collect()
    }

    /// Returns the free directory entries, counting `reused` as free.
    fn free_entries(&self, reused: &[usize]) -> Vec<usize> {
        (0..DIRECTORY_ENTRIES)
            .filter(|index| reused.contains(index) || self.entry(*index)[entry::USER] == FREE_ENTRY)
            .collect()
    }

    /// Returns the blocks that no entry uses, apart from the entries in
    /// `reused`.
    fn free_blocks(&self, reused: &[usize]) -> Vec<usize> {
        let mut used = [false; BLOCKS];
        used[..DIRECTORY_BLOCKS].fill(true);
        for index in (0..DIRECTORY_ENTRIES).filter(|index| !reused.contains(index)) {
            let entry = self.entry(index);
            if entry[entry::USER] == FREE_ENTRY {
                continue;
            }
            for &block in &entry[entry::BLOCKS..] {
                if let Some(used) = used.get_mut(usize::from(block)) {
                    *used = true;
                }
            }
        }
        (0..BLOCKS).filter(|&block| !used[block]).collect()
    }

    /// Returns directory entry `index`.
    fn entry(&self, index: usize) -> [u8; ENTRY_BYTES] {
        let record = self.read_record(index * ENTRY_BYTES / RECORD_BYTES);
        let start = index * ENTRY_BYTES % RECORD_BYTES;
        let mut entry = [0; ENTRY_BYTES];
        entry.copy_from_slice(&record[start..start + ENTRY_BYTES]);
        entry
    }

    /// Writes directory entry `index`.
    fn write_entry(&mut self, index: usize, entry: &[u8; ENTRY_BYTES]) {
        let number = index * ENTRY_BYTES / RECORD_BYTES;
        let start = index * ENTRY_BYTES % RECORD_BYTES;
        let mut record = self.read_record(number);
        record[start..start + ENTRY_BYTES].copy_from_slice(entry);
        self.write_record(number, &record);
    }

    /// Returns the data of record `number`.
    fn read_record(&self, number: usize) -> [u8; RECORD_BYTES] {
        let place = Place::of(number);
        let start = place.offset() + place.data_offset();
        let mut record = [0; RECORD_BYTES];
        record.copy_from_slice(&self.bytes[start..start + RECORD_BYTES]);
        record
    }

    /// Writes `data` into record `number`, with the header, the sum and the
    /// stop byte that the BIOS writes.
    fn write_record(&mut self, number: usize, data: &[u8; RECORD_BYTES]) {
        let place = Place::of(number);
        let sum = data.iter().fold(0, |sum: u8, byte| sum.wrapping_add(*byte));
        let mut sector = [0; SECTOR_BYTES];
        sector[offset::TRACK] = SYNC_BIT | place.track as u8;
        if place.track < DATA_FORMAT_TRACK {
            sector[offset::SYSTEM_WORD..offset::SYSTEM_DATA].copy_from_slice(&SYSTEM_WORD);
            sector[offset::SYSTEM_STOP] = STOP_BYTE;
            sector[offset::SYSTEM_SUM] = sum;
        } else {
            sector[offset::DATA_SECTOR] = place.sector;
            sector[offset::DATA_SUM] = sum;
            sector[offset::DATA_STOP] = STOP_BYTE;
        }
        let data_offset = place.data_offset();
        sector[data_offset..data_offset + RECORD_BYTES].copy_from_slice(data);

        let start = place.offset();
        self.bytes[start..start + SECTOR_BYTES].copy_from_slice(&sector);
    }
}

/// Where a CP/M record sits on the disk.
struct Place {
    /// The track.
    track: usize,
    /// The BIOS sector, before the Altair skew.
    sector: u8,
    /// The sector on the track, after the Altair skew.
    position: u8,
}

impl Place {
    /// Finds record `number`, counted from the first track after the
    /// reserved ones.
    fn of(number: usize) -> Self {
        let track = RESERVED_TRACKS + number / SECTORS;
        let sector = TRANSLATE[number % SECTORS];
        let position = if track >= DATA_FORMAT_TRACK && sector % 2 == 1 {
            sector ^ SKEW_BIT
        } else {
            sector
        };
        Self {
            track,
            sector,
            position,
        }
    }

    /// Returns the position of the sector in the image.
    fn offset(&self) -> usize {
        (self.track * SECTORS + usize::from(self.position)) * SECTOR_BYTES
    }

    /// Returns where the data starts in the sector.
    fn data_offset(&self) -> usize {
        if self.track < DATA_FORMAT_TRACK {
            offset::SYSTEM_DATA
        } else {
            offset::DATA_DATA
        }
    }
}

/// Turns a file name such as `cputest.com` into the 11 bytes of a CP/M
/// directory entry: `CPUTEST COM`.
fn file_name(name: &str) -> Result<[u8; NAME_BYTES], String> {
    let (base, kind) = name.rsplit_once('.').unwrap_or((name, ""));
    let valid = |part: &str, max: usize| {
        part.len() <= max
            && part
                .chars()
                .all(|c| c.is_ascii_graphic() && !FORBIDDEN_CHARACTERS.contains(c))
    };
    if base.is_empty() || !valid(base, 8) || !valid(kind, 3) {
        return Err(format!(
            "{name} is not a CP/M file name: up to eight letters or digits, a dot and up to three more"
        ));
    }
    let mut bytes = [b' '; NAME_BYTES];
    for (byte, letter) in bytes.iter_mut().zip(base.bytes()) {
        *byte = letter.to_ascii_uppercase();
    }
    for (byte, letter) in bytes[8..].iter_mut().zip(kind.bytes()) {
        *byte = letter.to_ascii_uppercase();
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns a disk with an empty directory.
    fn empty_disk() -> CpmDisk {
        let mut disk = CpmDisk::new(vec![0; DISK_BYTES]).unwrap();
        for index in 0..DIRECTORY_ENTRIES {
            disk.write_entry(index, &[FREE_ENTRY; ENTRY_BYTES]);
        }
        disk
    }

    /// Returns the entries in use, in directory order.
    fn used_entries(disk: &CpmDisk) -> Vec<[u8; ENTRY_BYTES]> {
        (0..DIRECTORY_ENTRIES)
            .map(|index| disk.entry(index))
            .filter(|entry| entry[entry::USER] != FREE_ENTRY)
            .collect()
    }

    /// Returns the sector of the image that holds record `number`.
    fn sector(disk: &CpmDisk, number: usize) -> &[u8] {
        let start = Place::of(number).offset();
        &disk.bytes[start..start + SECTOR_BYTES]
    }

    #[test]
    fn rejects_image_of_wrong_size() {
        let bytes = vec![0; DISK_BYTES - 1];

        let result = CpmDisk::new(bytes);

        assert!(result.is_err());
    }

    #[test]
    fn turns_file_name_into_directory_name() {
        let name = "cputest.com";

        let bytes = file_name(name);

        assert_eq!(bytes, Ok(*b"CPUTEST COM"));
    }

    #[test]
    fn pads_short_name_with_spaces() {
        let name = "a.c";

        let bytes = file_name(name);

        assert_eq!(bytes, Ok(*b"A       C  "));
    }

    #[test]
    fn rejects_long_or_odd_names() {
        let names = ["", "toolongname.com", "a.text", "a b.com", "a*.com"];

        let results = names.map(file_name);

        assert!(results.iter().all(Result::is_err), "{results:?}");
    }

    #[test]
    fn finds_record_on_system_track() {
        let number = 37;

        let place = Place::of(number);

        assert_eq!((place.track, place.sector, place.position), (3, 10, 10));
    }

    #[test]
    fn skews_odd_sector_on_data_track() {
        // Record 144 is CP/M sector 16 of track 6: BIOS sector 1.
        let number = 4 * SECTORS + 16;

        let place = Place::of(number);

        assert_eq!((place.track, place.sector, place.position), (6, 1, 17));
    }

    #[test]
    fn writes_system_sector_as_bios_does() {
        let mut disk = empty_disk();
        let data = [0x01; RECORD_BYTES];

        disk.write_record(0, &data);

        let sector = sector(&disk, 0);
        assert_eq!(sector[..3], [0x82, 0x00, 0x01]);
        assert_eq!(sector[3..131], data);
        assert_eq!(sector[131..133], [0xFF, 0x80]);
    }

    #[test]
    fn writes_data_sector_as_bios_does() {
        let mut disk = empty_disk();
        let data = [0x01; RECORD_BYTES];
        let number = 4 * SECTORS + 16;

        disk.write_record(number, &data);

        let sector = sector(&disk, number);
        assert_eq!(sector[..7], [0x86, 0x01, 0x00, 0x00, 0x80, 0x00, 0x00]);
        assert_eq!(sector[7..135], data);
        assert_eq!(sector[135], 0xFF);
    }

    #[test]
    fn put_makes_directory_entry() {
        let mut disk = empty_disk();

        disk.put("hello.com", &[0xC9; 300]).unwrap();

        let entries = used_entries(&disk);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0][..16], *b"\0HELLO   COM\0\0\0\x03");
        assert_eq!(entries[0][16..18], [2, 0]);
    }

    #[test]
    fn put_stores_data_padded_with_end_of_file() {
        let mut disk = empty_disk();

        disk.put("hello.com", &[0xC9; 130]).unwrap();

        let first = disk.read_record(2 * BLOCK_RECORDS);
        let second = disk.read_record(2 * BLOCK_RECORDS + 1);
        assert_eq!(first, [0xC9; RECORD_BYTES]);
        assert_eq!(second[..2], [0xC9, 0xC9]);
        assert!(second[2..].iter().all(|&byte| byte == END_OF_FILE_BYTE));
    }

    #[test]
    fn put_gives_each_16_kb_an_entry() {
        let mut disk = empty_disk();

        disk.put("big.dat", &vec![0; 20 * 1024]).unwrap();

        let entries = used_entries(&disk);
        let extents = entries.iter().map(|entry| (entry[entry::EXTENT], entry[entry::RECORDS]));
        assert_eq!(extents.collect::<Vec<_>>(), [(0, 128), (1, 32)]);
        assert_eq!(entries[0][16..24], [2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(entries[1][16..20], [10, 11, 0, 0]);
    }

    #[test]
    fn put_replaces_file_with_same_name() {
        let mut disk = empty_disk();
        disk.put("x.com", &[0x11; 4096]).unwrap();

        disk.put("X.COM", &[0x22; 10]).unwrap();

        let entries = used_entries(&disk);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0][entry::RECORDS], 1);
        assert_eq!(disk.read_record(2 * BLOCK_RECORDS)[0], 0x22);
    }

    #[test]
    fn put_fails_on_full_disk_and_keeps_it() {
        let mut disk = empty_disk();
        disk.put("a.dat", &vec![0; 290 * 1024]).unwrap();
        let before = disk.bytes.clone();

        let result = disk.put("b.dat", &[0; 8 * 1024]);

        assert!(result.is_err());
        assert!(disk.bytes == before);
    }

    #[test]
    fn put_fails_on_full_directory() {
        let mut disk = empty_disk();
        for index in 0..DIRECTORY_ENTRIES {
            disk.put(&format!("f{index}.txt"), &[]).unwrap();
        }

        let result = disk.put("one.txt", &[]);

        assert_eq!(result, Err("the directory is full".to_string()));
    }
}
