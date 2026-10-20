//! The `altair` command: runs the emulated Altair 8800 in your terminal.
//!
//! ```text
//!   altair image | basic | cpm             altair disk format | put | ls
//!              │                                      │
//!   boot::*::setup    read, check the files     disk::run and disk::cpm
//!              │
//!            Setup
//!              │
//!   machine::start    cards, toggle in, EXAMINE, RUN
//!              │
//!   host::clock::run  2 MHz until Ctrl+] or HLT
//! ```
//!
//! `main.rs` only calls [`run`], so the examples in this library run under
//! `cargo test`:
//!
//! ```
//! use clap::Parser;
//!
//! let disk = std::env::temp_dir().join(format!("app-doc-{}-lib.dsk", std::process::id()));
//! let args = app::cli::Args::parse_from(["altair", "disk", "format", disk.to_str().unwrap()]);
//!
//! app::run(&args)?;
//!
//! assert_eq!(std::fs::metadata(&disk)?.len(), 337_568);
//! # std::fs::remove_file(&disk)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod boot;
pub mod cli;
pub mod disk;
pub mod host;
pub mod machine;

use std::error::Error;

use crate::boot::{Setup, basic, cpm, disk_basic, image};
use crate::cli::{Args, Command};
use crate::host::clock::{self, Exit, Speed};
use crate::host::console::{Console, RawMode};

/// Carries out the command in `args`.
///
/// A boot command switches your terminal to raw mode and returns when you
/// quit or the CPU halts with interrupts disabled, then restores the terminal.
///
/// ```
/// use clap::Parser;
///
/// let disk = std::env::temp_dir().join(format!("app-doc-{}-run.dsk", std::process::id()));
/// let format = app::cli::Args::parse_from(["altair", "disk", "format", disk.to_str().unwrap()]);
/// let ls = app::cli::Args::parse_from(["altair", "disk", "ls", disk.to_str().unwrap()]);
///
/// app::run(&format)?;
/// app::run(&ls)?; // prints "0 files, 296 KB free"
/// # std::fs::remove_file(&disk)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    match &args.command {
        Command::Image { image } => boot(image::setup(image)?, args),
        Command::Basic { basic, tape } => boot(basic::setup(*basic, tape)?, args),
        Command::Cpm { prom, disks } => boot(cpm::setup(prom, disks)?, args),
        Command::DiskBasic { prom, disks } => boot(disk_basic::setup(prom, disks)?, args),
        Command::Disk(command) => disk::run(command),
    }
}

/// Starts the Altair as `setup` says, with the options in `args`, and runs
/// it until you quit or the CPU halts with interrupts disabled.
fn boot(mut setup: Setup, args: &Args) -> Result<(), Box<dyn Error>> {
    if let Some(switches) = args.switches {
        setup.sense_switches = switches;
    }
    let speed = if args.fast {
        Speed::Unlimited
    } else {
        Speed::Altair
    };

    let raw_mode = RawMode::enable()
        .map_err(|error| format!("can't switch the terminal to raw mode: {error}"))?;
    let (console, quit) = Console::start(&setup.tape, setup.erase_key);
    let mut altair = machine::start(setup, console)?;
    let exit = clock::run(&mut altair, &quit, speed);
    drop(raw_mode);

    if let Exit::Halted = exit {
        eprintln!("\naltair: the CPU halted");
    }
    Ok(())
}
