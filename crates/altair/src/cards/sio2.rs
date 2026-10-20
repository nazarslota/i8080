//! The MITS 88-2SIO serial card. [`Sio2`] documents it.

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
/// Status bit 7 (IRQ): an enabled interrupt condition requests service.
const INTERRUPT_REQUEST_BIT: u8 = 0x80;
/// Control bit 7: enables receive interrupts.
const RECEIVE_INTERRUPT_BIT: u8 = 0x80;
/// Control bits 6 and 5: select RTS and transmit interrupt behavior.
const TRANSMITTER_CONTROL_BITS: u8 = 0x60;
/// Control bits 6 and 5 equal 01: transmit interrupts enabled.
const TRANSMIT_INTERRUPT_BITS: u8 = 0x20;
/// Control bits 1 and 0 equal 11: holds the ACIA in master reset.
const MASTER_RESET_BITS: u8 = 0x03;
/// No status bits set.
const NO_BITS: u8 = 0b0000_0000;

/// What the data port returns when no character waits.
const EMPTY_DATA: u8 = 0x00;

/// The MITS 88-2SIO serial card: the link between the Altair and your
/// terminal.
///
/// The card has two serial ports, each built on a Motorola 6850 ACIA chip.
/// Altair BASIC talks to port A, and the emulation covers port A alone:
///
/// | Port  | `IN`                       | `OUT`                         |
/// |-------|----------------------------|-------------------------------|
/// | `10H` | the status byte            | a control byte                |
/// | `11H` | the character you typed    | a character for the screen    |
///
/// | Bit | Mask  | Set when                          |
/// |-----|-------|-----------------------------------|
/// | 0   | `01H` | a character waits in port `11H`   |
/// | 1   | `02H` | the card can take a new character |
/// | 7   | `80H` | an enabled interrupt needs service |
///
/// # Printing a character
///
/// A program waits for bit 1 and then writes the character. This
/// loop prints the character in register `B`:
///
/// ```text
/// 0200H  DBH 10H      WAIT: IN 10H    read the status
/// 0202H  E6H 02H            ANI 02H   keep bit 1
/// 0204H  CAH 00H 02H        JZ WAIT   bit 1 is 0: the card is busy
/// 0207H  78H                MOV A,B
/// 0208H  D3H 11H            OUT 11H   send the character
/// ```
///
/// # Reading a key
///
/// A program waits for bit 0 and then reads the character. This
/// loop leaves the key in register `A`:
///
/// ```text
/// 0300H  DBH 10H      WAIT: IN 10H    read the status
/// 0302H  E6H 01H            ANI 01H   keep bit 0
/// 0304H  CAH 00H 03H        JZ WAIT   bit 0 is 0: you typed nothing
/// 0307H  DBH 11H            IN 11H    A = the key
/// ```
///
/// The emulated card sends a character at once, so bit 1 stays set outside
/// master reset and the first loop never waits. Baud rate, character format,
/// modem signals and receive errors are not modeled.
///
/// # Interrupts
///
/// A program writes the control byte to `10H`. Bit 7 enables receive
/// interrupts; bits 6 and 5 equal to `01` enable transmit interrupts. Low
/// bits equal to `11` hold the ACIA in master reset, clear its receive
/// register and suppress interrupts until the program writes a setup byte.
/// For 8N1 at divide by 16, use `95H` for receive IRQ, `35H` for transmit
/// IRQ, `B5H` for both or `15H` for polling.
///
/// With receive interrupts enabled, [`Card::interrupt_pending`] checks the
/// terminal between CPU instructions. A read of `11H` clears the receive
/// request. Transmit IRQ stays active until the program disables it, since
/// transmission takes no emulated time. Reading status or acknowledging an
/// interrupt clears neither request.
///
/// The card connects to PINT. Without a vectored interrupt controller, the
/// Altair reads `FFH` from the open bus during acknowledge and runs `RST 7`
/// at `0038H`. [`Altair::step`](crate::Altair::step) carries the request to
/// the CPU. Polling programs keep fetching input through status reads, so
/// their tape readers do not run ahead of initialization.
///
/// The ports and the status bits come from the [Altair 8800 Monitor] source,
/// which defines them for the 88-2SIO, and from the bootstrap loader in
/// [Loading Basic with the 88-2SIO Board].
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
/// backplane.plug(Ram::new(0x0000..=0xFFFF)?);
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
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// The [88-2SIO manual], Software Options and Status Register sections,
/// defines the control and status bits. The [Altair Clone manual], section
/// F, describes PINT and the open-bus `RST 7` without an 88-VI/RTC board.
///
/// [88-2SIO manual]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Altair%202SIO%20Serial%20I-O%20searchable.pdf
/// [Altair Clone manual]: https://altairclone.com/downloads/manuals/Altair%20Clone%20Operator%27s%20Manual.pdf
/// [Altair 8800 Monitor]: https://k1.spdns.de/Develop/Projects/zxsp/zxsp.alt10/zasm/Test/8080/original/Altair8800_Monitor.z80
/// [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf
pub struct Sio2<T> {
    terminal: T,
    received: Option<u8>,
    control: u8,
}

impl<T: Terminal> Sio2<T> {
    /// Creates a card wired to `terminal`.
    ///
    /// ```
    /// use altair::cards::{Card, Sio2, Terminal};
    ///
    /// /// A keyboard with one key pressed and no screen.
    /// struct Key(Option<u8>);
    ///
    /// impl Terminal for Key {
    ///     fn receive(&mut self) -> Option<u8> {
    ///         self.0.take()
    ///     }
    ///
    ///     fn send(&mut self, _byte: u8) {}
    /// }
    ///
    /// let mut sio = Sio2::new(Key(Some(b'A')));
    ///
    /// assert_eq!(sio.input(0x10), Some(0b0000_0011)); // a key waits, ready to send
    /// assert_eq!(sio.input(0x11), Some(b'A'));
    ///
    /// let mut sio = Sio2::new(Key(Some(b'B')));
    /// sio.output(0x10, 0x95); // receive IRQ, 8N1, divide by 16
    ///
    /// assert!(sio.interrupt_pending()); // no status read needed
    /// assert_eq!(sio.input(0x10), Some(0x83)); // IRQ, TDRE, RDRF
    /// assert_eq!(sio.input(0x11), Some(b'B'));
    /// assert!(!sio.interrupt_pending());
    /// ```
    pub fn new(terminal: T) -> Self {
        Self {
            terminal,
            received: None,
            control: 0x00,
        }
    }

    /// Returns the status byte. Before it answers, the card takes the next
    /// key from the terminal if its data register is empty.
    fn status(&mut self) -> u8 {
        if self.is_reset() {
            return NO_BITS;
        }
        self.poll_receive();

        let received = if self.received.is_some() {
            RECEIVED_BIT
        } else {
            NO_BITS
        };
        let interrupt = if self.irq() {
            INTERRUPT_REQUEST_BIT
        } else {
            NO_BITS
        };
        received | READY_TO_SEND_BIT | interrupt
    }

    /// Takes one character if the receive register is empty.
    fn poll_receive(&mut self) {
        if self.received.is_none() {
            self.received = self.terminal.receive();
        }
    }

    /// Returns whether the control byte holds the ACIA in master reset.
    fn is_reset(&self) -> bool {
        self.control & MASTER_RESET_BITS == MASTER_RESET_BITS
    }

    /// Returns whether a receive or transmit condition requests service.
    fn irq(&self) -> bool {
        !self.is_reset()
            && ((self.control & RECEIVE_INTERRUPT_BIT != 0 && self.received.is_some())
                || self.control & TRANSMITTER_CONTROL_BITS == TRANSMIT_INTERRUPT_BITS)
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
        match port {
            STATUS_PORT => {
                self.control = value;
                if self.is_reset() {
                    self.received = None;
                }
            }
            DATA_PORT if !self.is_reset() => self.terminal.send(value),
            _ => {}
        }
    }

    fn interrupt_pending(&mut self) -> bool {
        if !self.is_reset() && self.control & RECEIVE_INTERRUPT_BIT != 0 {
            self.poll_receive();
        }
        self.irq()
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
    fn receive_interrupt_arrives_without_status_read() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));
        sio.output(0x10, 0x95); // receive IRQ, 8N1, divide by 16

        let pending = sio.interrupt_pending();
        let status = sio.input(0x10);
        let key = sio.input(0x11);
        let after_read = sio.interrupt_pending();

        assert!(pending);
        assert_eq!(status, Some(0x83)); // IRQ, TDRE, RDRF
        assert_eq!(key, Some(b'A'));
        assert!(!after_read);
        assert_eq!(sio.input(0x10), Some(0x02));
    }

    #[test]
    fn polling_mode_does_not_prefetch_tape_before_reset() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"AB"));

        let pending = sio.interrupt_pending();
        sio.output(0x10, 0x03); // master reset before the loader reads the tape
        sio.output(0x10, 0x15); // polling, 8N1, divide by 16
        let status = sio.input(0x10);
        let key = sio.input(0x11);

        assert!(!pending);
        assert_eq!(status, Some(0x03));
        assert_eq!(key, Some(b'A'));
    }

    #[test]
    fn receive_interrupt_waits_for_key_and_keeps_it_until_read() {
        let mut sio = Sio2::new(FakeTerminal::default());
        sio.output(0x10, 0x95);
        let before_key = sio.interrupt_pending();
        sio.terminal.keys.extend(b"AB");

        let first = sio.interrupt_pending();
        let repeated = sio.interrupt_pending();
        let key = sio.input(0x11);
        let next = sio.interrupt_pending();

        assert!(!before_key);
        assert!(first);
        assert!(repeated);
        assert_eq!(key, Some(b'A'));
        assert!(next);
        assert_eq!(sio.input(0x11), Some(b'B'));
    }

    #[test]
    fn masking_receive_interrupt_keeps_waiting_character() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));
        sio.output(0x10, 0x95);
        sio.interrupt_pending();

        sio.output(0x10, 0x15); // disable receive IRQ
        let masked = sio.interrupt_pending();
        let status = sio.input(0x10);
        sio.output(0x10, 0x95); // enable it without losing the character
        let enabled = sio.interrupt_pending();

        assert!(!masked);
        assert_eq!(status, Some(0x03));
        assert!(enabled);
        assert_eq!(sio.input(0x11), Some(b'A'));
    }

    #[test]
    fn transmit_interrupt_uses_both_transmitter_control_bits() {
        let cases = [
            (0x15, false), // RTS low, TX IRQ off
            (0x35, true),  // RTS low, TX IRQ on
            (0x55, false), // RTS high, TX IRQ off
            (0x75, false), // break, TX IRQ off
        ];

        for (control, expected) in cases {
            let mut sio = Sio2::new(FakeTerminal::default());

            sio.output(0x10, control);
            let pending = sio.interrupt_pending();
            let status = sio.input(0x10);

            assert_eq!(pending, expected, "control {control:#04X}");
            assert_eq!(status, Some(if expected { 0x82 } else { 0x02 }));
        }
    }

    #[test]
    fn transmit_interrupt_stays_set_after_instant_transmission() {
        let mut sio = Sio2::new(FakeTerminal::default());
        sio.output(0x10, 0x35);

        sio.output(0x11, b'A');
        let pending = sio.interrupt_pending();
        sio.output(0x10, 0x15);
        let disabled = sio.interrupt_pending();

        assert_eq!(sio.terminal.screen, b"A");
        assert!(pending);
        assert!(!disabled);
    }

    #[test]
    fn master_reset_clears_received_data_and_interrupts() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"AB"));
        sio.output(0x10, 0xB5); // receive and transmit IRQ on
        sio.interrupt_pending(); // A waits in the receive register

        sio.output(0x10, 0xA3); // low bits 11 reset despite the IRQ enable bits
        let pending = sio.interrupt_pending();
        let status = sio.input(0x10);
        let data = sio.input(0x11);
        sio.output(0x10, 0x95);
        let after_setup = sio.interrupt_pending();

        assert!(!pending);
        assert_eq!(status, Some(0x00));
        assert_eq!(data, Some(0x00));
        assert!(after_setup);
        assert_eq!(sio.input(0x11), Some(b'B'));
    }

    #[test]
    fn ignores_other_ports() {
        let mut sio = Sio2::new(FakeTerminal::with_keys(b"A"));

        sio.output(0x12, b'A');

        assert_eq!(sio.input(0x12), None);
        assert!(sio.terminal.screen.is_empty());
    }
}
