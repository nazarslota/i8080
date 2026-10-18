//! `altair`: runs the emulated Altair 8800 in your terminal.
//!
//! ```text
//! cargo run --release -- <image.bin>
//! cargo run --release -- --tape "roms/8K BASIC Ver 4-0.tap" --basic 8k-4.0
//! ```
//!
//! The program loads a memory image at 0000H, or the MITS bootstrap loader
//! with a paper tape in the reader, and starts the CPU at 0000H. Press
//! Ctrl+] to quit.

mod args;
mod console;
mod loader;

use std::path::Path;
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::{fs, thread};

use altair::Altair;
use altair::cards::{Cpu, Ram, Sio2};
use clap::Parser;

use crate::args::Args;
use crate::console::{Console, RawMode};
use crate::loader::{Basic, bootstrap_loader};

/// How many times per second the emulator checks the clock. Between two
/// checks it runs the CPU for one [`SLICE`].
const SLICES_PER_SECOND: u64 = 100;
/// How long one slice takes on the real Altair.
const SLICE: Duration = Duration::from_nanos(1_000_000_000 / SLICES_PER_SECOND);
/// The cycles the real Altair executes in one [`SLICE`].
const SLICE_CYCLES: u64 = Cpu::CLOCK_HZ / SLICES_PER_SECOND;
/// The sense switches for a memory image when you don't set them: all down.
const SWITCHES_DOWN: u8 = 0x00;

/// What the machine holds when it starts.
struct Setup {
    /// The bytes at 0000H, where the CPU starts.
    memory: Vec<u8>,
    /// The paper tape in the reader. Empty when you load a memory image.
    tape: Vec<u8>,
    /// The sense switches A8 to A15 that the program reads.
    sense_switches: u8,
}

/// How a run of the emulator ended.
enum Exit {
    /// You pressed Ctrl+].
    Quit,
    /// The program executed `HLT`.
    Halted,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match prepare(&args).and_then(run) {
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

/// Reads the files from the command line and decides what goes into memory
/// and into the tape reader.
fn prepare(args: &Args) -> Result<Setup, String> {
    if let Some(path) = &args.tape {
        let basic = args.basic.expect("clap requires --basic with --tape");
        let tape = read(path)?;
        if tape.first() != Some(&basic.leader()) {
            return Err(format!(
                "{}: not a {basic} tape, which starts with {:02X}H",
                path.display(),
                basic.leader()
            ));
        }
        return Ok(Setup {
            memory: bootstrap_loader(basic).to_vec(),
            tape,
            sense_switches: args.switches.unwrap_or(Basic::SENSE_SWITCHES),
        });
    }

    let path = args
        .image
        .as_ref()
        .expect("clap requires an image or --tape");
    let memory = read(path)?;
    if memory.len() > 0x10000 {
        return Err(format!(
            "{}: the image is larger than 64 KB",
            path.display()
        ));
    }
    Ok(Setup {
        memory,
        tape: Vec::new(),
        sense_switches: args.switches.unwrap_or(SWITCHES_DOWN),
    })
}

/// Reads a file and puts its path into the error message.
fn read(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// Runs the machine at the Altair's speed until you quit or the CPU halts.
/// Restores the terminal before it returns.
fn run(setup: Setup) -> Result<Exit, String> {
    let _raw_mode = RawMode::enable()
        .map_err(|error| format!("can't switch the terminal to raw mode: {error}"))?;
    let (console, quit) = Console::start(setup.tape);
    let mut altair = Altair::builder()
        .plug(Cpu::new()) // The 88-CPU card
        .plug(Ram::new(0x0000..=0x7FFF)) // First 32 KB card
        .plug(Ram::new(0x8000..=0xFFFF)) // Second 32 KB card
        .plug(Sio2::new(console)) // The 88-2SIO card at ports 10H and 11H
        .build();

    // What an Altair owner did by hand: toggle the program in at 0000H,
    // examine 0000H again, set the sense switches and press RUN.
    let mut panel = altair.front_panel();
    panel.toggle_in(0x0000, &setup.memory);
    panel.set_switches(0x0000);
    panel.examine();
    panel.set_switches(u16::from(setup.sense_switches) << 8);
    panel.run();

    let mut next_slice = Instant::now();
    loop {
        if quit.load(Ordering::Relaxed) {
            return Ok(Exit::Quit);
        }

        let mut cycles = 0;
        while cycles < SLICE_CYCLES {
            if altair.is_halted() {
                return Ok(Exit::Halted);
            }
            cycles += u64::from(altair.step());
        }

        // Sleep off the time the real Altair would still need for the slice.
        next_slice += SLICE;
        match next_slice.checked_duration_since(Instant::now()) {
            Some(wait) => thread::sleep(wait),
            None => next_slice = Instant::now(),
        }
    }
}
