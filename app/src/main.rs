//! `altair`: runs the emulated Altair 8800 in your terminal.
//!
//! ```text
//! cargo run --release -- image roms/hello.bin
//! cargo run --release -- tape 8k-4.0 "roms/8K BASIC Ver 4-0.tap"
//! cp roms/cpm22b23-56k.dsk cpm.dsk
//! cargo run --release -- disk roms/DBL.HEX cpm.dsk
//! cargo run --release -- put cpm.dsk roms/CPUTEST.COM
//! ```
//!
//! The program reads the files, builds the Altair for them, starts it from
//! the front panel and runs it at 2 MHz. Press Ctrl+] to quit. `put` copies
//! files onto a CP/M disk and doesn't start the Altair.

mod args;
mod boot;
mod clock;
mod console;
mod cpm_disk;
mod disk_file;
mod machine;

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;

use crate::args::{Args, BootCommand, Command};
use crate::boot::{Boot, in_file};
use crate::clock::{Exit, Speed};
use crate::console::{Console, RawMode};
use crate::cpm_disk::CpmDisk;

fn main() -> ExitCode {
    let args = Args::parse();
    let result = match &args.command {
        Command::Boot(command) => run(command, &args),
        Command::Put { disk, files } => put(disk, files),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("altair: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Boots the Altair as `command` says, with the options in `args`, and runs
/// it until you quit or the CPU halts. Restores the terminal before it
/// returns.
fn run(command: &BootCommand, args: &Args) -> Result<(), Box<dyn Error>> {
    let boot = Boot::load(command)?;
    let sense_switches = args.switches.unwrap_or(boot.sense_switches());
    let speed = if args.fast { Speed::Unlimited } else { Speed::Altair };

    let raw_mode = RawMode::enable()
        .map_err(|error| format!("can't switch the terminal to raw mode: {error}"))?;
    let (console, quit) = Console::start(boot.tape(), boot.erase_key());
    let mut altair = machine::start(boot, console, sense_switches)?;
    let exit = clock::run(&mut altair, &quit, speed);
    drop(raw_mode);

    if let Exit::Halted = exit {
        eprintln!("\naltair: the CPU halted");
    }
    Ok(())
}

/// Copies `files` onto the CP/M disk at `disk`. Writes the image only when
/// all of them fit.
fn put(disk: &Path, files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let mut cpm = CpmDisk::new(boot::read(disk)?).map_err(|error| in_file(disk, error))?;
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| in_file(file, "the name is not a CP/M file name"))?;
        cpm.put(name, &boot::read(file)?)
            .map_err(|error| in_file(file, error))?;
    }
    fs::write(disk, cpm.into_bytes()).map_err(|error| in_file(disk, error))?;
    Ok(())
}
