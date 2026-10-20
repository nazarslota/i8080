//! Altair DOS 1.0 from an 8-inch floppy.
//!
//! DOS uses the DBL PROM and the 88-DCDD controller. Sense switches `10H`
//! select the 88-2SIO terminal with one stop bit. Backspace sends RUBOUT,
//! which DOS echoes with the deleted character between backslashes.
//! The [DOS manual], sections 1-2 and 1-5, describes startup and input.
//!
//! ```
//! let prom = std::env::temp_dir().join(format!("app-doc-{}-dos.hex", std::process::id()));
//! std::fs::write(&prom, ":01FF0000C33D\n:00000001FF\n")?;
//! let disk = std::env::temp_dir().join(format!("app-doc-{}-dos.dsk", std::process::id()));
//! std::fs::write(&disk, vec![0; altair::cards::DISK_BYTES])?;
//!
//! let setup = app::boot::dos::setup(&prom, &[disk.clone()])?;
//!
//! assert_eq!(setup.start, 0xFF00);
//! assert_eq!(setup.sense_switches, 0x10);
//! assert_eq!(setup.erase_key, 0x7F);
//! # std::fs::remove_file(&prom)?;
//! # std::fs::remove_file(&disk)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! [DOS manual]: https://altairclone.com/downloads/manuals/Altair%20DOS%20User%27s%20Manual.pdf

use std::error::Error;
use std::path::{Path, PathBuf};

use super::{Setup, floppy};
use crate::host::console::RUBOUT_KEY;

/// The drives that DOS 1.0 accepts at startup, 0 through 5.
pub(crate) const DRIVE_COUNT: usize = 6;
/// A12 up: the 88-2SIO terminal with one stop bit.
const SENSE_SWITCHES_BYTE: u8 = 0x10;

/// Opens the DBL PROM and one to six disks for Altair DOS.
///
/// The first disk must hold DOS. The emulator opens the images for writing,
/// so use copies. Backspace sends RUBOUT. See the module example.
pub fn setup(prom: &Path, disks: &[PathBuf]) -> Result<Setup, Box<dyn Error>> {
    floppy::setup(prom, disks, DRIVE_COUNT, SENSE_SWITCHES_BYTE, RUBOUT_KEY)
}
