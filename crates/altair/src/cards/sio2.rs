//! The MITS 88-2SIO serial card: the link between the Altair and your
//! terminal.
//!
//! The Altair has no keyboard and no screen. You plug a terminal into a
//! serial card, and programs talk to the card through two I/O ports:
//!
//! | Port  | `IN`                       | `OUT`                         |
//! |-------|----------------------------|-------------------------------|
//! | `10H` | the status byte            | a control byte (ignored here) |
//! | `11H` | the character you typed    | a character for the screen    |
//!
//! Two bits of the status byte matter:
//!
//! | Bit | Mask  | Set when                          |
//! |-----|-------|-----------------------------------|
//! | 0   | `01H` | a character waits in port `11H`   |
//! | 1   | `02H` | the card can take a new character |
//!
//! # Printing a character
//!
//! A program waits until bit 1 is set and then writes the character. This
//! loop prints the character in register `B`:
//!
//! ```text
//! 0200H  DBH 10H      WAIT: IN 10H    read the status
//! 0202H  E6H 02H            ANI 02H   keep bit 1
//! 0204H  CAH 00H 02H        JZ WAIT   bit 1 is 0: the card is busy
//! 0207H  78H                MOV A,B
//! 0208H  D3H 11H            OUT 11H   send the character
//! ```
//!
//! # Reading a key
//!
//! A program waits until bit 0 is set and then reads the character. This
//! loop leaves the key in register `A`:
//!
//! ```text
//! 0300H  DBH 10H      WAIT: IN 10H    read the status
//! 0302H  E6H 01H            ANI 01H   keep bit 0
//! 0304H  CAH 00H 03H        JZ WAIT   bit 0 is 0: you typed nothing
//! 0307H  DBH 11H            IN 11H    A = the key
//! ```
//!
//! On the real card the first loop waits while the card shifts the previous
//! character out, one bit at a time. The emulated card sends at once, so
//! the loop never waits.
//!
//! The ports and the status bits come from the [Altair 8800 Monitor] source,
//! which defines them for the 88-2SIO, and from the bootstrap loader in
//! [Loading Basic with the 88-2SIO Board].
//!
//! [Altair 8800 Monitor]: https://k1.spdns.de/Develop/Projects/zxsp/zxsp.alt10/zasm/Test/8080/original/Altair8800_Monitor.z80
//! [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf

use super::Card;

/// The port that returns the status byte on `IN` and takes control bytes on
/// `OUT`.
const STATUS_PORT: u8 = 0x10;
/// The port that returns the received character on `IN` and sends a
/// character on `OUT`.
const DATA_PORT: u8 = 0x11;

/// Status bit 0 (RDRF, receive data register full): a character waits in
/// the data port.
const RECEIVED_BIT: u8 = 0b0000_0001;
/// Status bit 1 (TDRE, transmit data register empty): the card can take a
/// character to send.
const READY_TO_SEND_BIT: u8 = 0b0000_0010;
/// No status bits set.
const NO_BITS: u8 = 0b0000_0000;

/// What the data port returns when no character waits.
const EMPTY_DATA: u8 = 0x00;

/// The MITS 88-2SIO serial card, port A at `10H` and `11H`.
///
/// The card has two serial ports, each built on a Motorola 6850 ACIA chip.
/// Altair BASIC talks to the first one, port A, and this emulation covers
/// port A alone.
///
/// A program polls the status port and checks two bits:
///
/// - bit 0: a character waits, so `IN 11H` returns it;
/// - bit 1: the card can send, so `OUT 11H` sends a character.
///
/// The emulated card sends a character at once, so bit 1 stays set. The card
/// ignores the control bytes that programs write to the status port: they
/// set the speed and the character format, and a cable to your terminal
/// needs neither.
///
/// ```
/// use altair::Backplane;
/// use altair::cards::{Ram, Sio2, Terminal};
/// use i8080::{Cpu, MemoryBus};
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
/// let mut backplane = Backplane::new();
/// backplane.plug(Ram::new(0x0000..=0xFFFF));
/// backplane.plug(Sio2::new(Screen));
///
/// // 0000H  3EH 41H  MVI A,41H
/// // 0002H  D3H 11H  OUT 11H    prints A
/// // 0004H  76H      HLT
/// for (addr, byte) in (0..).zip([0x3E, 0x41, 0xD3, 0x11, 0x76]) {
///     backplane.write(addr, byte);
/// }
///
/// let mut cpu = Cpu::new();
/// while !cpu.is_halted() {
///     cpu.step(&mut backplane);
/// }
/// ```
pub struct Sio2<T> {
    terminal: T,
    received: Option<u8>,
}

impl<T: Terminal> Sio2<T> {
    /// Creates a card wired to `terminal`.
    pub fn new(terminal: T) -> Self {
        Self {
            terminal,
            received: None,
        }
    }

    /// Returns the status byte. Before it answers, the card takes the next
    /// key from the terminal if its data register is empty.
    fn status(&mut self) -> u8 {
        if self.received.is_none() {
            self.received = self.terminal.receive();
        }

        let received = if self.received.is_some() {
            RECEIVED_BIT
        } else {
            NO_BITS
        };
        received | READY_TO_SEND_BIT
    }

    /// Returns the received character and empties the data register. With
    /// nothing received, it returns [`EMPTY_DATA`].
    fn take_received(&mut self) -> u8 {
        self.received.take().unwrap_or(EMPTY_DATA)
    }
}

impl<T: Terminal> Card for Sio2<T> {
    fn input(&mut self, port: u8) -> Option<u8> {
        match port {
            STATUS_PORT => Some(self.status()),
            DATA_PORT => Some(self.take_received()),
            _ => None,
        }
    }

    fn output(&mut self, port: u8, value: u8) {
        if port == DATA_PORT {
            self.terminal.send(value);
        }
    }
}

/// The other end of the serial cable: the terminal you type on.
///
/// The [`Sio2`] sends what the program prints to the terminal and takes
/// the keys you type from it. The `altair` crate does no host I/O, so the
/// application implements this trait and connects it to your real keyboard
/// and screen.
pub trait Terminal {
    /// Returns the next key you typed, or `None` if you typed nothing.
    fn receive(&mut self) -> Option<u8>;

    /// Shows `byte` on the screen.
    fn send(&mut self, byte: u8);
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    /// A terminal with keys typed in advance that records what the card
    /// sends.
    #[derive(Default)]
    struct FakeTerminal {
        keys: VecDeque<u8>,
        screen: Vec<u8>,
    }

    impl FakeTerminal {
        fn with_keys(keys: &[u8]) -> Self {
            Self {
                keys: keys.iter().copied().collect(),
                screen: Vec::new(),
            }
        }
    }

    impl Terminal for FakeTerminal {
        fn receive(&mut self) -> Option<u8> {
            self.keys.pop_front()
        }

        fn send(&mut self, byte: u8) {
            self.screen.push(byte);
        }
    }

    #[test]
    fn status_reports_ready_to_send_without_keys() {
        let mut sio = Sio2::new(FakeTerminal::default());

        let status = sio.input(0x10);

        assert_eq!(status, Some(0b0000_0010));
    }

    #[test]
    fn status_reports_waiting_key() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));

        let status = sio.input(0x10);

        assert_eq!(status, Some(0b0000_0011));
    }

    #[test]
    fn data_port_returns_key_after_status() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"AB"));
        sio.input(0x10);

        let key = sio.input(0x11);

        assert_eq!(key, Some(b'A'));
        assert_eq!(sio.input(0x10), Some(0b0000_0011));
        assert_eq!(sio.input(0x11), Some(b'B'));
    }

    #[test]
    fn repeated_status_keeps_waiting_key() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"AB"));
        sio.input(0x10);

        sio.input(0x10);

        assert_eq!(sio.input(0x11), Some(b'A'));
    }

    #[test]
    fn reading_data_empties_register() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));
        sio.input(0x10);
        sio.input(0x11);

        let status = sio.input(0x10);

        assert_eq!(status, Some(0b0000_0010));
    }

    #[test]
    fn data_port_sends_byte_to_terminal() {
        let mut sio = Sio2::new(FakeTerminal::default());

        sio.output(0x11, b'A');

        assert_eq!(sio.terminal.screen, b"A");
    }

    #[test]
    fn status_port_ignores_control_byte() {
        let mut sio = Sio2::new(FakeTerminal::default());

        sio.output(0x10, 0x03);

        assert!(sio.terminal.screen.is_empty());
        assert_eq!(sio.input(0x10), Some(0b0000_0010));
    }

    #[test]
    fn ignores_other_ports() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));

        sio.output(0x12, b'A');

        assert_eq!(sio.input(0x12), None);
        assert!(sio.terminal.screen.is_empty());
    }
}
