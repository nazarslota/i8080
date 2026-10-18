use i8080::{Cpu, MemoryBus};

use crate::{Backplane, Ram, Sio2, Terminal};

/// An Altair 8800 with two 32 KB memory cards and an 88-2SIO serial card
/// wired to your terminal: the machine that runs Altair BASIC.
///
/// ```
/// use altair::{Altair, Terminal};
///
/// /// A terminal that prints to the standard output and has no keyboard.
/// struct Screen;
///
/// impl Terminal for Screen {
///     fn receive(&mut self) -> Option<u8> {
///         None
///     }
///
///     fn send(&mut self, byte: u8) {
///         print!("{}", char::from(byte));
///     }
/// }
///
/// let mut altair = Altair::new(Screen);
/// // 0000H  3EH 41H  MVI A,41H
/// // 0002H  D3H 11H  OUT 11H    prints A
/// // 0004H  76H      HLT
/// altair.load(0x0000, &[0x3E, 0x41, 0xD3, 0x11, 0x76]);
///
/// while !altair.is_halted() {
///     altair.step();
/// }
/// ```
pub struct Altair {
    cpu: Cpu,
    backplane: Backplane,
}

impl Altair {
    /// The clock frequency: the Altair's 8080 runs at 2 MHz, so it executes
    /// 2 million cycles per second.
    pub const CLOCK_HZ: u64 = 2_000_000;

    /// Creates the machine with every byte of RAM at 0 and `PC` at 0000H.
    pub fn new(terminal: impl Terminal + 'static) -> Self {
        let mut backplane = Backplane::new();
        backplane.plug(Ram::new(0x0000..=0x7FFF)); // First 32 KB card
        backplane.plug(Ram::new(0x8000..=0xFFFF)); // Second 32 KB card
        backplane.plug(Sio2::new(terminal)); // The 88-2SIO card at port 11H

        Self {
            cpu: Cpu::new(),
            backplane,
        }
    }

    /// Writes `bytes` to memory from `address` up, through the backplane.
    ///
    /// The real Altair had no such shortcut. You set each byte on the front
    /// panel switches and pressed DEPOSIT, or the loader read the program
    /// from a paper tape. The emulator doesn't model the front panel yet, so
    /// it writes the bytes into the memory cards directly.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` run past the end of memory at FFFFH.
    pub fn load(&mut self, address: u16, bytes: &[u8]) {
        let end = usize::from(address) + bytes.len();
        assert!(end <= 0x10000, "the bytes run past the end of memory");

        for (addr, &byte) in (address..=u16::MAX).zip(bytes) {
            self.backplane.write(addr, byte);
        }
    }

    /// Executes one instruction and returns how many clock cycles it took.
    pub fn step(&mut self) -> u32 {
        self.cpu.step(&mut self.backplane)
    }

    /// Returns `true` after a `HLT` instruction stopped the CPU.
    pub const fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    /// A terminal that records what the machine sends. The test keeps a
    /// second handle to the screen, because the machine owns the terminal.
    struct SharedScreen(Rc<RefCell<Vec<u8>>>);

    impl Terminal for SharedScreen {
        fn receive(&mut self) -> Option<u8> {
            None
        }

        fn send(&mut self, byte: u8) {
            self.0.borrow_mut().push(byte);
        }
    }

    fn altair_with_screen() -> (Altair, Rc<RefCell<Vec<u8>>>) {
        let screen = Rc::new(RefCell::new(Vec::new()));
        let altair = Altair::new(SharedScreen(Rc::clone(&screen)));
        (altair, screen)
    }

    fn run_until_halted(altair: &mut Altair) {
        while !altair.is_halted() {
            altair.step();
        }
    }

    #[test]
    fn runs_program_that_prints_to_terminal() {
        let (mut altair, screen) = altair_with_screen();
        // MVI A,41H; OUT 11H; MVI A,42H; OUT 11H; HLT
        altair.load(
            0x0000,
            &[0x3E, 0x41, 0xD3, 0x11, 0x3E, 0x42, 0xD3, 0x11, 0x76],
        );

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"AB");
    }

    #[test]
    fn load_places_bytes_at_address() {
        let (mut altair, screen) = altair_with_screen();
        // 0000H: LDA 2000H; OUT 11H; HLT
        altair.load(0x0000, &[0x3A, 0x00, 0x20, 0xD3, 0x11, 0x76]);
        altair.load(0x2000, b"Z");

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"Z");
    }

    #[test]
    fn load_reaches_last_address() {
        let (mut altair, screen) = altair_with_screen();
        // 0000H: LDA FFFFH; OUT 11H; HLT
        altair.load(0x0000, &[0x3A, 0xFF, 0xFF, 0xD3, 0x11, 0x76]);
        altair.load(0xFFFF, b"Z");

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"Z");
    }

    #[test]
    fn load_crosses_from_first_card_to_second() {
        let (mut altair, screen) = altair_with_screen();
        // 0000H: LDA 7FFFH; OUT 11H; LDA 8000H; OUT 11H; HLT
        altair.load(
            0x0000,
            &[
                0x3A, 0xFF, 0x7F, 0xD3, 0x11, 0x3A, 0x00, 0x80, 0xD3, 0x11, 0x76,
            ],
        );
        altair.load(0x7FFF, b"YZ");

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"YZ");
    }

    #[test]
    #[should_panic(expected = "past the end of memory")]
    fn load_rejects_bytes_past_end_of_memory() {
        let (mut altair, _) = altair_with_screen();

        altair.load(0xFFFF, &[0x11, 0x22]);
    }
}
