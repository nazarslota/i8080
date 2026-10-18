//! A memory image: the bytes of a program as they sit in memory, with no
//! header or format. The owner toggles them in at 0000H with the front
//! panel switches and presses RUN there.
//!
//! ```
//! let image = std::env::temp_dir().join(format!("app-doc-{}-image.bin", std::process::id()));
//! std::fs::write(&image, [0x3E, 0x41, 0x76])?; // MVI A,41H; HLT
//!
//! let setup = app::boot::image::setup(&image)?;
//!
//! let program = setup.toggle_in.unwrap();
//! assert_eq!((program.start, program.bytes), (0x0000, vec![0x3E, 0x41, 0x76]));
//! # std::fs::remove_file(&image)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use std::path::Path;

use super::{Program, SWITCHES_DOWN, Setup};
use crate::host::console::RUBOUT_KEY;
use crate::host::{in_file, read};

/// Where the owner toggles in the image and starts it.
const IMAGE_ADDRESS: u16 = 0x0000;
/// The largest memory image: the 64 KB that the 8080 addresses.
const IMAGE_MAX_BYTES: usize = 0x10000;

/// Reads the image at `path` and sets the Altair up to run it.
///
/// ```
/// let image = std::env::temp_dir().join(format!("app-doc-{}-image-setup.bin", std::process::id()));
/// std::fs::write(&image, vec![0; 0x10001])?;
///
/// let setup = app::boot::image::setup(&image);
///
/// assert!(setup.is_err()); // one byte past 64 KB
/// # std::fs::remove_file(&image)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn setup(path: &Path) -> Result<Setup, Box<dyn Error>> {
    let image = read(path)?;
    if image.len() > IMAGE_MAX_BYTES {
        return Err(in_file(path, "the image is larger than 64 KB").into());
    }
    Ok(Setup {
        toggle_in: Some(Program {
            start: IMAGE_ADDRESS,
            bytes: image,
        }),
        prom: None,
        disks: Vec::new(),
        tape: Vec::new(),
        start: IMAGE_ADDRESS,
        sense_switches: SWITCHES_DOWN,
        erase_key: RUBOUT_KEY,
    })
}
