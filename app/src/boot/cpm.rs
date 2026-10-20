//! CP/M 2.2 from floppy disks.
//!
//! The owner put the MITS disk boot loader (DBL) into a PROM at FF00H,
//! inserted the CP/M disk in drive 0 and more disks in drives 1 to 3,
//! examined FF00H and pressed RUN. DBL reads CP/M from drive 0, and CP/M
//! calls the drives A: to D:.
//!
//! ```
//! let prom = std::env::temp_dir().join(format!("app-doc-{}-cpm.hex", std::process::id()));
//! std::fs::write(&prom, ":01FF0000C33D\n:00000001FF\n")?; // C3H at FF00H
//! let disk = std::env::temp_dir().join(format!("app-doc-{}-cpm.dsk", std::process::id()));
//! std::fs::write(&disk, vec![0; altair::cards::DISK_BYTES])?;
//!
//! let setup = app::boot::cpm::setup(&prom, &[disk.clone()])?;
//!
//! assert_eq!(setup.start, 0xFF00);
//! assert_eq!(setup.disks.len(), 1);
//! # std::fs::remove_file(&prom)?;
//! # std::fs::remove_file(&disk)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use std::path::{Path, PathBuf};

use super::{SWITCHES_DOWN, Setup, floppy};
use crate::host::console::BACKSPACE_KEY;

/// The drives that the CP/M BIOS knows, A: to D:.
pub(crate) const DRIVE_COUNT: usize = 4;

/// Opens the DBL PROM and one to four disks for CP/M.
///
/// The first disk must hold CP/M. The emulator opens the images for writing,
/// so use copies. Backspace sends Ctrl+H. See the module example.
pub fn setup(prom: &Path, disks: &[PathBuf]) -> Result<Setup, Box<dyn Error>> {
    floppy::setup(prom, disks, DRIVE_COUNT, SWITCHES_DOWN, BACKSPACE_KEY)
}
