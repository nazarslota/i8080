//! Keeps the emulated Altair at the speed of the real one.
//!
//! Your computer runs the 8080 faster than 2 MHz. So the clock runs the CPU
//! for the cycles of one slice, then sleeps until the slice is up:
//!
//! ```text
//!   │◄──────────── 10 ms ────────────►│◄──────────── 10 ms ───────────►│
//!   │ CPU: 20,000 cycles │   sleep    │ CPU: 20,000 cycles │   sleep   │
//! ```
//!
//! With [`Speed::Unlimited`] the clock skips the sleep, and a diagnostic
//! that takes hours on the real Altair ends in seconds.
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//!
//! use altair::Altair;
//! use altair::cards::Ram;
//! use app::host::clock::{self, Exit, Speed};
//!
//! let mut altair = Altair::builder().plug(Ram::new(0x0000..=0xFFFF)?).build();
//! let mut panel = altair.front_panel();
//! panel.toggle_in(0x0000, &[0x76]); // HLT
//! panel.set_switches(0x0000);
//! panel.examine();
//! panel.run();
//!
//! let exit = clock::run(&mut altair, &AtomicBool::new(false), Speed::Unlimited);
//!
//! assert!(matches!(exit, Exit::Halted));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use altair::Altair;

/// How many times per second the clock checks the time. Between two checks
/// it runs the CPU for one [`SLICE`].
const SLICES_PER_SECOND: u64 = 100;
/// How long one slice takes on the real Altair.
const SLICE: Duration = Duration::from_nanos(1_000_000_000 / SLICES_PER_SECOND);
/// The cycles the real Altair executes in one [`SLICE`].
const SLICE_CYCLES: u64 = Altair::CLOCK_HZ / SLICES_PER_SECOND;

/// How fast the clock runs the CPU.
#[derive(Clone, Copy)]
pub enum Speed {
    /// 2 MHz, as on the real Altair.
    Altair,
    /// As fast as your computer can.
    Unlimited,
}

/// How a run of the emulator ended.
pub enum Exit {
    /// You pressed Ctrl+].
    Quit,
    /// The program executed `HLT`.
    Halted,
}

/// Runs `altair` at `speed` until `quit` turns `true` or the CPU halts.
///
/// ```
/// use std::sync::atomic::AtomicBool;
///
/// use altair::Altair;
/// use altair::cards::Ram;
/// use app::host::clock::{self, Exit, Speed};
///
/// let mut altair = Altair::builder().plug(Ram::new(0x0000..=0xFFFF)?).build();
/// altair.front_panel().run(); // NOP after NOP from 0000H
///
/// let exit = clock::run(&mut altair, &AtomicBool::new(true), Speed::Altair);
///
/// assert!(matches!(exit, Exit::Quit)); // the flag says you pressed Ctrl+]
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run(altair: &mut Altair, quit: &AtomicBool, speed: Speed) -> Exit {
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

        if let Speed::Unlimited = speed {
            continue;
        }
        // Sleep off the time the real Altair would still need for the slice.
        next_slice += SLICE;
        match next_slice.checked_duration_since(Instant::now()) {
            Some(wait) => thread::sleep(wait),
            None => next_slice = Instant::now(),
        }
    }
}
