//! A disk image file as the [`Disk`] in the boot drive.
//!
//! An image holds the bytes of an 8-inch floppy in order: track 0 sector 0
//! first, then sector 1, up to sector 31 of track 76. Each sector takes 137
//! bytes, so a full image takes 337,664 bytes. The 88-DCDD finds a byte by
//! its track, its sector and its place in the sector:
//!
//! ```text
//! offset = (track * 32 + sector) * 137 + byte
//! ```
//!
//! The card asks [`DiskFile`] for that offset, and the file answers from a
//! copy in memory:
//!
//! ```text
//!         IN 0AH                   OUT 0AH
//!           │                        │
//!   Dcdd: read(offset)     Dcdd: write(offset, byte)
//!           │                        │
//!           ▼                        ▼
//!      ┌──────────────── DiskFile ────────────────┐
//!      │  bytes: Vec<u8>  ◄── reads and writes    │
//!      │  the image file  ◄── writes, at once     │
//!      └──────────────────────────────────────────┘
//! ```
//!
//! The emulator changes the image you pass to `altair disk`: when CP/M
//! saves a file, the bytes land in the image before the next instruction.
//! Keep a copy of the original. [`DiskFile::open`] fails on a read-only
//! image and on one too short for a disk, so you learn about it at start,
//! before CP/M tries to read or write.

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

use altair::cards::{DISK_BYTES, Disk};

/// A disk image in a file, which keeps every write the program makes.
///
/// The disk reads from a copy in memory and writes each byte both to the
/// copy and to the file at once, as a real floppy keeps what the head
/// writes. So you lose nothing when the emulator stops.
pub struct DiskFile {
    bytes: Vec<u8>,
    file: File,
}

impl DiskFile {
    /// Opens the image at `path` for reading and writing. Fails when the
    /// image holds less than a disk. An image can run past the disk: the
    /// one on deramp.com pads it to a multiple of 256 bytes.
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        if bytes.len() < DISK_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "not an 8-inch Altair disk: it holds {} bytes, short of {DISK_BYTES}",
                    bytes.len()
                ),
            ));
        }
        Ok(Self { bytes, file })
    }

    /// Writes `byte` to the file at `offset`.
    fn write_to_file(&mut self, offset: usize, byte: u8) -> io::Result<()> {
        let position = u64::try_from(offset).map_err(io::Error::other)?;
        self.file.seek(SeekFrom::Start(position))?;
        self.file.write_all(&[byte])
    }
}

impl Disk for DiskFile {
    fn read(&mut self, offset: usize) -> u8 {
        Disk::read(&mut self.bytes, offset)
    }

    fn write(&mut self, offset: usize, byte: u8) {
        Disk::write(&mut self.bytes, offset, byte);
        // A program can't react to a failed write, so we drop the error, as
        // the console does with a broken screen.
        let _ = self.write_to_file(offset, byte);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    /// A file in the temporary directory that the test removes when done.
    struct TempFile(PathBuf);

    impl TempFile {
        fn with(name: &str, bytes: &[u8]) -> Self {
            let path = std::env::temp_dir().join(format!("altair-{}-{name}", std::process::id()));
            fs::write(&path, bytes).unwrap();
            Self(path)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// Returns a blank disk image with `bytes` at its start.
    fn image(bytes: &[u8]) -> Vec<u8> {
        let mut image = vec![0; DISK_BYTES];
        image[..bytes.len()].copy_from_slice(bytes);
        image
    }

    #[test]
    fn reads_bytes_of_file() {
        let file = TempFile::with("reads", &image(&[0x11, 0x22]));
        let mut disk = DiskFile::open(&file.0).unwrap();

        let bytes = [disk.read(0), disk.read(1), disk.read(2)];

        assert_eq!(bytes, [0x11, 0x22, 0x00]);
    }

    #[test]
    fn write_reaches_file_at_once() {
        let file = TempFile::with("writes", &image(&[0x11, 0x22, 0x33]));
        let mut disk = DiskFile::open(&file.0).unwrap();

        disk.write(1, 0x99);

        assert_eq!(fs::read(&file.0).unwrap()[..3], [0x11, 0x99, 0x33]);
        assert_eq!(disk.read(1), 0x99);
    }

    #[test]
    fn open_rejects_image_shorter_than_disk() {
        let file = TempFile::with("short", &[0x11; 100]);

        let disk = DiskFile::open(&file.0);

        assert!(disk.is_err());
    }
}
