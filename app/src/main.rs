//! `altair`: runs the emulated Altair 8800 in your terminal.
//!
//! ```text
//! cargo run --release -- image roms/hello.bin
//! cargo run --release -- tape 8k-4.0 "roms/8K BASIC Ver 4-0.tap"
//! cp roms/cpm22b23-56k.dsk cpm.dsk
//! cargo run --release -- disk roms/DBL.HEX cpm.dsk
//! ```
//!
//! The program reads the files, builds the Altair for them, starts it from
//! the front panel and runs it at 2 MHz. Press Ctrl+] to quit.

mod args;
mod boot;
mod clock;
mod console;
mod disk_file;
mod machine;

use std::error::Error;
use std::process::ExitCode;

use clap::Parser;

use crate::args::Args;
use crate::boot::Boot;
use crate::clock::Exit;
use crate::console::{Console, RawMode};

fn main() -> ExitCode {
    match run(&Args::parse()) {
        Ok(Exit::Quit) => ExitCode::SUCCESS,
        Ok(Exit::Halted) => {
            eprintln!("\naltair: the CPU halted");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("altair: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Boots the Altair as `args` say and runs it until you quit or the CPU
/// halts. Restores the terminal before it returns.
fn run(args: &Args) -> Result<Exit, Box<dyn Error>> {
    let boot = Boot::load(&args.command)?;
    let sense_switches = args.switches.unwrap_or(boot.sense_switches());

    let _raw_mode = RawMode::enable()
        .map_err(|error| format!("can't switch the terminal to raw mode: {error}"))?;
    let (console, quit) = Console::start(boot.tape(), boot.erase_key());
    let mut altair = machine::start(boot, console, sense_switches)?;
    Ok(clock::run(&mut altair, &quit))
}
