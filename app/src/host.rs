//! The emulator's side of your computer: the terminal, the clock and the
//! files. The libraries do no I/O, so everything that touches the host
//! lives here.
//!
//! ```
//! use std::path::Path;
//!
//! let error = app::host::read(Path::new("no-such.dsk")).unwrap_err();
//!
//! assert!(error.starts_with("no-such.dsk: "));
//! ```

pub mod clock;
pub mod console;
pub mod disk_file;

use std::path::Path;
use std::{fmt, fs};

/// Reads a file and puts its path into the error message.
///
/// ```
/// let file = std::env::temp_dir().join(format!("app-doc-{}-read.bin", std::process::id()));
/// std::fs::write(&file, [0x76])?;
///
/// let bytes = app::host::read(&file)?;
///
/// assert_eq!(bytes, [0x76]);
/// # std::fs::remove_file(&file)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| in_file(path, error))
}

/// Puts `path` in front of `error`.
///
/// ```
/// use std::path::Path;
///
/// let message = app::host::in_file(Path::new("a.dsk"), "the disk is full");
///
/// assert_eq!(message, "a.dsk: the disk is full");
/// ```
pub fn in_file(path: &Path, error: impl fmt::Display) -> String {
    format!("{}: {error}", path.display())
}
