//! Altair Disk BASIC from an 8-inch floppy.
//!
//! Disk BASIC uses the same DBL PROM and 88-DCDD as CP/M. Its sense
//! switches differ: A12 is up (`10H`), selecting the 88-2SIO terminal with
//! one stop bit. The [Altair Clone manual], Disk BASIC demonstration,
//! describes this setup for version 300-5-F.
//!
//! ```
//! let prom = std::env::temp_dir().join(format!("app-doc-{}-disk-basic.hex", std::process::id()));
//! std::fs::write(&prom, ":01FF0000C33D\n:00000001FF\n")?;
//! let disk = std::env::temp_dir().join(format!("app-doc-{}-disk-basic.dsk", std::process::id()));
//! std::fs::write(&disk, vec![0; altair::cards::DISK_BYTES])?;
//!
//! let setup = app::boot::disk_basic::setup(&prom, &[disk.clone()])?;
//!
//! assert_eq!(setup.start, 0xFF00);
//! assert_eq!(setup.sense_switches, 0x10);
//! assert_eq!(setup.disks.len(), 1);
//! # std::fs::remove_file(&prom)?;
//! # std::fs::remove_file(&disk)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [Altair Clone manual]: https://altairclone.com/downloads/manuals/Altair%20Clone%20Operator%27s%20Manual.pdf

use std::error::Error;
use std::path::{Path, PathBuf};

use super::{Setup, floppy};
use crate::host::console::BACKSPACE_KEY;

/// The physical drives that Disk BASIC can address, 0 through 15.
pub(crate) const DRIVE_COUNT: usize = 16;
/// A12 up: the 88-2SIO terminal with one stop bit.
const SENSE_SWITCHES_BYTE: u8 = 0x10;

/// Opens the DBL PROM and one to sixteen disks for Disk BASIC.
///
/// The first disk must hold Disk BASIC. The emulator opens the images for
/// writing, so use copies. Backspace sends Ctrl+H. See the module example.
pub fn setup(prom: &Path, disks: &[PathBuf]) -> Result<Setup, Box<dyn Error>> {
    floppy::setup(prom, disks, DRIVE_COUNT, SENSE_SWITCHES_BYTE, BACKSPACE_KEY)
}
