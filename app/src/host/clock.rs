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
//! that takes hours on the real Altair ends in seconds. With a [`Trace`],
//! the clock records each instruction before the CPU runs it.
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//!
//! use altair::Altair;
//! use altair::cards::Ram;
//! use app::host::clock::{self, Exit, Speed};
//! use app::trace::Trace;
//!
//! let mut altair = Altair::builder().plug(Ram::new(0x0000..=0xFFFF)?).build();
//! let mut panel = altair.front_panel();
//! panel.toggle_in(0x0000, &[0x76]); // HLT
//! panel.set_switches(0x0000);
//! panel.examine();
//! panel.run();
//!
//! let mut trace = Trace::new(10);
//!
//! let exit = clock::run(&mut altair, &AtomicBool::new(false), Speed::Unlimited, Some(&mut trace));
//!
//! assert!(matches!(exit, Exit::Halted));
//! let mut file = Vec::new();
//! trace.write_to(&mut file)?;
//! assert!(String::from_utf8(file)?.contains("0000  76        HLT"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use altair::Altair;

use crate::trace::Trace;

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
    /// The program executed `HLT` with CPU interrupts disabled.
    Halted,
}

/// Runs `altair` at `speed` until you quit or the CPU halts with INTE clear,
/// and adds each instruction to `trace` when you pass one.
///
/// With INTE set, `HLT` waits for a card's interrupt instead of ending the
/// run. The clock keeps stepping the machine so the cards can receive input.
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
/// let exit = clock::run(&mut altair, &AtomicBool::new(true), Speed::Altair, None);
///
/// assert!(matches!(exit, Exit::Quit)); // the flag says you pressed Ctrl+]
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run(
    altair: &mut Altair,
    quit: &AtomicBool,
    speed: Speed,
    mut trace: Option<&mut Trace>,
) -> Exit {
    let mut next_slice = Instant::now();
    loop {
        if quit.load(Ordering::Relaxed) {
            return Exit::Quit;
        }

        let mut cycles = 0;
        while cycles < SLICE_CYCLES {
            if altair.is_halted() && !altair.interrupts_enabled() {
                return Exit::Halted;
            }
            cycles += u64::from(match trace.as_deref_mut() {
                Some(trace) => {
                    let (cycles, line) = altair.trace_step();
                    if let Some(line) = line {
                        trace.push(line);
                    }
                    cycles
                }
                None => altair.step(),
            });
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use altair::cards::{Ram, Sio2, Terminal};

    use super::*;

    /// A terminal that delivers A on its third interrupt sample, after HLT.
    struct DelayedKey {
        samples: usize,
    }

    impl Terminal for DelayedKey {
        fn receive(&mut self) -> Option<u8> {
            self.samples += 1;
            (self.samples == 3).then_some(b'A')
        }

        fn send(&mut self, _byte: u8) {}
    }

    /// A terminal that asks the clock to quit after the CPU reaches HLT.
    struct QuitWhileHalted {
        samples: usize,
        quit: Arc<AtomicBool>,
    }

    impl Terminal for QuitWhileHalted {
        fn receive(&mut self) -> Option<u8> {
            self.samples += 1;
            if self.samples == 3 {
                self.quit.store(true, Ordering::Relaxed);
            }
            None
        }

        fn send(&mut self, _byte: u8) {}
    }

    /// Builds a program that enables RX IRQ, waits in HLT, then disables
    /// CPU interrupts and halts after the handler returns.
    fn waiting_machine(terminal: impl Terminal + 'static) -> Altair {
        let mut altair = Altair::builder()
            .plug(Ram::new(0x0000..=0xFFFF).expect("the full address range is valid"))
            .plug(Sio2::new(terminal))
            .build();
        let mut panel = altair.front_panel();
        // LXI SP,F000H; MVI A,95H; OUT 10H; EI; HLT; DI; HLT
        panel.toggle_in(
            0x0100,
            &[
                0x31, 0x00, 0xF0, 0x3E, 0x95, 0xD3, 0x10, 0xFB, 0x76, 0xF3, 0x76,
            ],
        );
        // RST 7 handler: IN 11H; STA 2000H; EI; RET
        panel.toggle_in(0x0038, &[0xDB, 0x11, 0x32, 0x00, 0x20, 0xFB, 0xC9]);
        panel.set_switches(0x0100);
        panel.examine();
        panel.run();
        altair
    }

    #[test]
    fn clock_runs_interrupt_handler_after_hlt() {
        let mut altair = waiting_machine(DelayedKey { samples: 0 });

        let exit = run(&mut altair, &AtomicBool::new(false), Speed::Unlimited, None);
        let interrupts_enabled = altair.interrupts_enabled();
        let mut panel = altair.front_panel();
        panel.reset(); // release HLT before the panel feeds its EXAMINE instruction
        panel.set_switches(0x2000);
        let received = panel.examine();

        assert!(matches!(exit, Exit::Halted));
        assert_eq!(received, b'A');
        assert!(!interrupts_enabled);
    }

    #[test]
    fn clock_traces_interrupt_but_not_hlt_wait() {
        let mut altair = waiting_machine(DelayedKey { samples: 0 });
        let mut trace = Trace::new(100);

        run(
            &mut altair,
            &AtomicBool::new(false),
            Speed::Unlimited,
            Some(&mut trace),
        );

        let mut file = Vec::new();
        trace.write_to(&mut file).unwrap();
        let text = String::from_utf8(file).unwrap();
        let instructions: Vec<&str> = text
            .lines()
            .skip(1)
            .map(|line| line[16..31].trim())
            .collect();
        assert_eq!(
            instructions,
            [
                "LXI SP,0F000H",
                "MVI A,95H",
                "OUT 10H",
                "EI",
                "HLT",
                "RST 7",
                "IN 11H",
                "STA 2000H",
                "EI",
                "RET",
                "DI",
                "HLT",
            ],
        );
    }

    #[test]
    fn clock_can_quit_while_hlt_waits_for_interrupt() {
        let quit = Arc::new(AtomicBool::new(false));
        let mut altair = waiting_machine(QuitWhileHalted {
            samples: 0,
            quit: Arc::clone(&quit),
        });

        let exit = run(&mut altair, &quit, Speed::Unlimited, None);

        assert!(matches!(exit, Exit::Quit));
        assert!(altair.is_halted());
        assert!(altair.interrupts_enabled());
    }
}
