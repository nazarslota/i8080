//! `altair`: runs the emulated Altair 8800 in your terminal.
//!
//! ```text
//! cargo run --release -p app -- <image.bin>
//! ```
//!
//! The program loads the memory image at 0000H and starts the CPU there.
//! Press Ctrl+] to quit.

mod console;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use std::{fs, thread};

use altair::Altair;
use clap::Parser;

use crate::console::{Console, RawMode};

/// How many times per second the emulator checks the clock. Between two
/// checks it runs the CPU for one [`SLICE`].
const SLICES_PER_SECOND: u64 = 100;
/// How long one slice takes on the real Altair.
const SLICE: Duration = Duration::from_nanos(1_000_000_000 / SLICES_PER_SECOND);
/// The cycles the real Altair executes in one [`SLICE`].
const SLICE_CYCLES: u64 = Altair::CLOCK_HZ / SLICES_PER_SECOND;

/// Runs the emulated Altair 8800 in your terminal. Press Ctrl+] to quit.
#[derive(Parser)]
#[command(name = "altair", version)]
struct Args {
    /// The memory image to load at 0000H. The CPU starts there.
    image: PathBuf,
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

    match run(&args.image) {
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

/// Loads the image and runs the machine at the Altair's speed until you quit
/// or the CPU halts. Restores the terminal before it returns.
fn run(path: &Path) -> Result<Exit, String> {
    let image = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if image.len() > 0x10000 {
        return Err(format!(
            "{}: the image is larger than 64 KB",
            path.display()
        ));
    }

    let _raw_mode = RawMode::enable()
        .map_err(|error| format!("can't switch the terminal to raw mode: {error}"))?;
    let (console, quit) = Console::start();
    let mut altair = Altair::new(console);
    altair.load(0x0000, &image);

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
