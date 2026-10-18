//! Keeps the emulated Altair at the speed of the real one.
//!
//! Your computer runs the 8080 faster than 2 MHz. So the clock runs the CPU
//! for the cycles of one slice, 10 ms on the real Altair, and sleeps until
//! those 10 ms are up.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use altair::Altair;
use altair::cards::Cpu;

/// How many times per second the clock checks the time. Between two checks
/// it runs the CPU for one [`SLICE`].
const SLICES_PER_SECOND: u64 = 100;
/// How long one slice takes on the real Altair.
const SLICE: Duration = Duration::from_nanos(1_000_000_000 / SLICES_PER_SECOND);
/// The cycles the real Altair executes in one [`SLICE`].
const SLICE_CYCLES: u64 = Cpu::CLOCK_HZ / SLICES_PER_SECOND;

/// How a run of the emulator ended.
pub enum Exit {
    /// You pressed Ctrl+].
    Quit,
    /// The program executed `HLT`.
    Halted,
}

/// Runs `altair` at the Altair's speed until `quit` turns `true` or the CPU
/// halts.
pub fn run(altair: &mut Altair, quit: &AtomicBool) -> Exit {
    let mut next_slice = Instant::now();
    loop {
        if quit.load(Ordering::Relaxed) {
            return Exit::Quit;
        }

        let mut cycles = 0;
        while cycles < SLICE_CYCLES {
            if altair.is_halted() {
                return Exit::Halted;
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
