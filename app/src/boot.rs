//! What you bring to the Altair to boot it: a memory image, a paper tape or
//! a floppy disk with its boot loader.

mod hex;
mod tape;

use std::error::Error;
use std::path::Path;
use std::{fmt, fs};

pub use hex::Program;
pub use tape::{Basic, bootstrap_loader};

use crate::args::Command;
use crate::disk_file::DiskFile;
use crate::machine::PROM_ADDRESS;

/// The sense switches for an image or a disk when you don't set them: all
/// down.
const SWITCHES_DOWN: u8 = 0x00;
/// The largest memory image: the 64 KB that the 8080 addresses.
const IMAGE_MAX_BYTES: usize = 0x10000;

/// How the Altair boots, with the files read and checked.
pub enum Boot {
    /// A memory image to toggle in at 0000H.
    Image(Vec<u8>),
    /// A BASIC tape for the reader, read by the bootstrap loader for
    /// `basic`.
    Tape { basic: Basic, tape: Vec<u8> },
    /// A floppy disk for drive 0, read by the boot loader in the PROM.
    Disk { loader: Program, disk: DiskFile },
}

impl Boot {
    /// Reads the files that `command` names and checks that they fit.
    pub fn load(command: &Command) -> Result<Self, Box<dyn Error>> {
        match command {
            Command::Image { image } => load_image(image),
            Command::Tape { basic, tape } => load_tape(*basic, tape),
            Command::Disk { loader, disk } => load_disk(loader, disk),
        }
    }

    /// Returns the paper tape for the reader. Empty unless you boot from a
    /// tape.
    pub fn tape(&self) -> &[u8] {
        match self {
            Self::Tape { tape, .. } => tape,
            Self::Image(_) | Self::Disk { .. } => &[],
        }
    }

    /// Returns the sense switches that the program expects.
    pub const fn sense_switches(&self) -> u8 {
        match self {
            Self::Tape { .. } => Basic::SENSE_SWITCHES,
            Self::Image(_) | Self::Disk { .. } => SWITCHES_DOWN,
        }
    }
}

/// Reads the memory image at `path`.
fn load_image(path: &Path) -> Result<Boot, Box<dyn Error>> {
    let image = read(path)?;
    if image.len() > IMAGE_MAX_BYTES {
        return Err(format!("{}: the image is larger than 64 KB", path.display()).into());
    }
    Ok(Boot::Image(image))
}

/// Reads the tape at `path` and checks that it holds `basic`.
fn load_tape(basic: Basic, path: &Path) -> Result<Boot, Box<dyn Error>> {
    let tape = read(path)?;
    if tape.first() != Some(&basic.leader()) {
        return Err(format!(
            "{}: not a {basic} tape, which starts with {:02X}H",
            path.display(),
            basic.leader()
        )
        .into());
    }
    Ok(Boot::Tape { basic, tape })
}

/// Reads the boot loader at `loader`, checks that it fits the PROM, and
/// opens the disk at `disk` for reading and writing.
fn load_disk(loader: &Path, disk: &Path) -> Result<Boot, Box<dyn Error>> {
    let text = fs::read_to_string(loader).map_err(|error| in_file(loader, error))?;
    let program = hex::parse(&text).map_err(|error| in_file(loader, error))?;
    if program.start < PROM_ADDRESS {
        return Err(in_file(
            loader,
            format!(
                "the loader starts at {:04X}H, below the PROM at {PROM_ADDRESS:04X}H",
                program.start
            ),
        )
        .into());
    }
    Ok(Boot::Disk {
        loader: program,
        disk: DiskFile::open(disk).map_err(|error| in_file(disk, error))?,
    })
}

/// Reads a file and puts its path into the error message.
fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| in_file(path, error))
}

/// Puts `path` in front of `error`.
fn in_file(path: &Path, error: impl fmt::Display) -> String {
    format!("{}: {error}", path.display())
}
