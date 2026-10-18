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
use std::fs;
use std::path::{Path, PathBuf};

use super::{SWITCHES_DOWN, Setup, hex};
use crate::host::console::BACKSPACE_KEY;
use crate::host::disk_file::DiskFile;
use crate::host::in_file;
use crate::machine::PROM_ADDRESS;

/// Reads the boot loader at `prom`, checks that it fits the PROM, opens the
/// disks at `disks` for reading and writing, and sets the Altair up to boot
/// from the first one. CP/M erases a character on Ctrl+H.
///
/// ```
/// let prom = std::env::temp_dir().join(format!("app-doc-{}-cpm-low.hex", std::process::id()));
/// std::fs::write(&prom, ":01000000C33C\n:00000001FF\n")?; // C3H at 0000H
///
/// let setup = app::boot::cpm::setup(&prom, &[]);
///
/// assert!(setup.is_err()); // the loader sits below the PROM
/// # std::fs::remove_file(&prom)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn setup(prom: &Path, disks: &[PathBuf]) -> Result<Setup, Box<dyn Error>> {
    let text = fs::read_to_string(prom).map_err(|error| in_file(prom, error))?;
    let loader = hex::parse(&text).map_err(|error| in_file(prom, error))?;
    if loader.start < PROM_ADDRESS {
        let error = format!(
            "the loader starts at {:04X}H, below the PROM at {PROM_ADDRESS:04X}H",
            loader.start
        );
        return Err(in_file(prom, error).into());
    }
    let disks = disks
        .iter()
        .map(|disk| DiskFile::open(disk).map_err(|error| in_file(disk, error)))
        .collect::<Result<_, _>>()?;
    Ok(Setup {
        toggle_in: None,
        start: loader.start,
        prom: Some(loader),
        disks,
        tape: Vec::new(),
        sense_switches: SWITCHES_DOWN,
        erase_key: BACKSPACE_KEY,
    })
}
