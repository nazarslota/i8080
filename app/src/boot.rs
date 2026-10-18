//! What the Altair owner brought and did before pressing RUN: toggled in a
//! memory image, put a BASIC tape in the reader, or inserted CP/M disks
//! with the boot loader in a PROM.
//!
//! Each way to boot reads and checks its files into a [`Setup`], before
//! your terminal switches to raw mode. [`crate::machine::start`] then puts
//! the setup into an Altair:
//!
//! | Boot    | `toggle_in`               | `prom`      | `disks`  | `tape`   | `start` | `erase_key` |
//! |---------|---------------------------|-------------|----------|----------|---------|-------------|
//! | `image` | the image at 0000H        |             |          |          | 0000H   | RUBOUT      |
//! | `basic` | the bootstrap loader      |             |          | the tape | 0000H   | RUBOUT      |
//! | `cpm`   |                           | DBL, FF00H  | A: to D: |          | FF00H   | Ctrl+H      |
//!
//! ```
//! let image = std::env::temp_dir().join(format!("app-doc-{}-boot.bin", std::process::id()));
//! std::fs::write(&image, [0x76])?; // HLT
//!
//! let setup = app::boot::image::setup(&image)?;
//!
//! assert_eq!(setup.start, 0x0000);
//! assert!(setup.prom.is_none() && setup.disks.is_empty());
//! # std::fs::remove_file(&image)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod basic;
pub mod cpm;
mod hex;
pub mod image;

use crate::host::disk_file::DiskFile;

/// The sense switches when a program doesn't need them: all down.
const SWITCHES_DOWN: u8 = 0x00;

/// A program for the Altair's memory: its bytes in one run from `start` up.
#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    /// The address of the first byte.
    pub start: u16,
    /// The bytes of the program.
    pub bytes: Vec<u8>,
}

/// The Altair as its owner set it up for one way to boot.
pub struct Setup {
    /// The program the owner toggles in with the front panel switches.
    pub toggle_in: Option<Program>,
    /// The PROM card, with the program it holds.
    pub prom: Option<Program>,
    /// The disks for drives 0 and up. Without disks the Altair gets no
    /// 88-DCDD.
    pub disks: Vec<DiskFile>,
    /// The paper tape in the reader, empty without one.
    pub tape: Vec<u8>,
    /// Where RUN starts the CPU.
    pub start: u16,
    /// The sense switches A8 to A15 that the program expects.
    pub sense_switches: u8,
    /// The byte that the program erases a character on, for your Backspace
    /// key.
    pub erase_key: u8,
}
