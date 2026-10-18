//! The machine: [`Altair`] and the builder that assembles it.

use i8080::{Cpu, IoBus, MemoryBus};

mod backplane;
mod front_panel;

pub use backplane::Backplane;
pub use front_panel::FrontPanel;

use crate::cards::Card;

/// The port where a program reads the sense switches, the upper eight front
/// panel switches.
const SENSE_SWITCHES_PORT: u8 = 0xFF;

/// An Altair 8800: the CPU, the front panel and the cards you plug in.
///
/// You assemble the machine with [`Altair::builder`] and operate it through
/// its [`FrontPanel`], as an Altair owner did:
///
/// ```
/// use altair::Altair;
/// use altair::cards::{Ram, Sio2, Terminal};
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
/// let mut altair = Altair::builder()
///     .plug(Ram::new(0x0000..=0xFFFF)?)
///     .plug(Sio2::new(Screen))
///     .build();
///
/// // 0000H  3EH 41H  MVI A,41H
/// // 0002H  D3H 11H  OUT 11H    prints A
/// // 0004H  76H      HLT
/// let mut panel = altair.front_panel();
/// panel.toggle_in(0x0000, &[0x3E, 0x41, 0xD3, 0x11, 0x76]);
/// panel.set_switches(0x0000);
/// panel.examine();
/// panel.run();
///
/// while !altair.is_halted() {
///     altair.step();
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Altair {
    cpu: Cpu,
    backplane: Backplane,
    /// The 16 front panel switches A0 to A15, a 1 bit for a switch up.
    switches: u16,
    /// `true` after RUN, `false` after STOP.
    running: bool,
}

impl Altair {
    /// The clock frequency of the 88-CPU card: it runs the 8080 at 2 MHz,
    /// 2 million cycles per second.
    pub const CLOCK_HZ: u64 = 2_000_000;

    /// Starts an Altair with the CPU card in place and the other slots
    /// empty.
    ///
    /// ```
    /// let altair = altair::Altair::builder().build(); // the CPU card alone
    ///
    /// assert!(!altair.is_running());
    /// ```
    pub fn builder() -> AltairBuilder {
        AltairBuilder {
            backplane: Backplane::new(),
        }
    }

    /// Returns the front panel, the switches you operate the machine with.
    ///
    /// ```
    /// # let mut altair = altair::Altair::builder().plug(altair::cards::Ram::new(0x0000..=0xFFFF)?).build();
    /// let mut panel = altair.front_panel();
    /// panel.set_switches(0x2000);
    ///
    /// assert_eq!(panel.examine(), 0x00); // the byte at 2000H
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn front_panel(&mut self) -> FrontPanel<'_> {
        FrontPanel::new(self)
    }

    /// Executes one instruction and returns how many clock cycles it took.
    ///
    /// `step` executes whether the front panel says RUN or STOP: you decide
    /// when to call it. Check [`is_running`](Altair::is_running) first to
    /// follow the panel.
    ///
    /// ```
    /// # let mut altair = altair::Altair::builder().plug(altair::cards::Ram::new(0x0000..=0xFFFF)?).build();
    /// altair.front_panel().toggle_in(0x0000, &[0x3E, 0x41]); // MVI A,41H
    /// altair.front_panel().reset();
    ///
    /// assert_eq!(altair.step(), 7); // MVI takes 7 cycles
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn step(&mut self) -> u32 {
        let [sense_switches, _] = self.switches.to_be_bytes();
        let mut bus = MachineBus {
            backplane: &mut self.backplane,
            sense_switches,
        };
        self.cpu.step(&mut bus)
    }

    /// Returns `true` after RUN on the front panel, `false` after STOP.
    /// [`FrontPanel::run`] shows an example.
    pub const fn is_running(&self) -> bool {
        self.running
    }

    /// Returns `true` after a `HLT` instruction stopped the CPU.
    ///
    /// ```
    /// # let mut altair = altair::Altair::builder().plug(altair::cards::Ram::new(0x0000..=0xFFFF)?).build();
    /// altair.front_panel().toggle_in(0x0000, &[0x76]); // HLT
    /// altair.front_panel().reset();
    ///
    /// altair.step();
    ///
    /// assert!(altair.is_halted());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }
}

/// Collects the cards for an [`Altair`], slot by slot.
pub struct AltairBuilder {
    backplane: Backplane,
}

impl AltairBuilder {
    /// Plugs `card` into the next free slot.
    ///
    /// ```
    /// use altair::Altair;
    /// use altair::cards::Ram;
    ///
    /// let mut altair = Altair::builder().plug(Ram::new(0x0000..=0x0FFF)?).build(); // 4 KB
    /// let mut panel = altair.front_panel();
    /// panel.set_switches(0x2000);
    ///
    /// assert_eq!(panel.examine(), 0xFF); // no card answers at 2000H
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn plug(mut self, card: impl Card + 'static) -> Self {
        self.backplane.plug(card);
        self
    }

    /// Builds the machine: every switch down, the CPU stopped, `PC` at 0000H.
    /// [`AltairBuilder::plug`] shows an example.
    pub fn build(self) -> Altair {
        Altair {
            cpu: Cpu::new(),
            backplane: self.backplane,
            switches: 0x0000,
            running: false,
        }
    }
}

/// The bus the CPU sees: the backplane, plus the front panel, which answers
/// `IN 0FFH` with the sense switches. The panel sits outside the slots, so
/// the machine wires it in here.
struct MachineBus<'a> {
    backplane: &'a mut Backplane,
    sense_switches: u8,
}

impl MemoryBus for MachineBus<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        self.backplane.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.backplane.write(addr, value);
    }
}

impl IoBus for MachineBus<'_> {
    fn input(&mut self, port: u8) -> u8 {
        if port == SENSE_SWITCHES_PORT {
            self.sense_switches
        } else {
            self.backplane.input(port)
        }
    }

    fn output(&mut self, port: u8, value: u8) {
        self.backplane.output(port, value);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::cards::{Ram, Sio2, Terminal};

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

    /// Builds an Altair with two 32 KB memory cards and an 88-2SIO wired to
    /// a screen the test can read.
    fn altair_with_screen() -> (Altair, Rc<RefCell<Vec<u8>>>) {
        let screen = Rc::new(RefCell::new(Vec::new()));
        let altair = Altair::builder()
            .plug(Ram::new(0x0000..=0x7FFF).unwrap())
            .plug(Ram::new(0x8000..=0xFFFF).unwrap())
            .plug(Sio2::new(SharedScreen(Rc::clone(&screen))))
            .build();
        (altair, screen)
    }

    /// Toggles `bytes` in from `address` up, then examines 0000H, so the CPU
    /// starts there.
    fn toggle_in(altair: &mut Altair, address: u16, bytes: &[u8]) {
        let mut panel = altair.front_panel();
        panel.toggle_in(address, bytes);
        panel.set_switches(0x0000);
        panel.examine();
    }

    fn run_until_halted(altair: &mut Altair) {
        while !altair.is_halted() {
            altair.step();
        }
    }

    #[test]
    fn build_stops_cpu_at_0000h() {
        let (altair, _) = altair_with_screen();

        assert!(!altair.is_running());
        assert_eq!(altair.cpu.registers.pc, 0x0000);
    }

    #[test]
    fn runs_program_that_prints_to_terminal() {
        let (mut altair, screen) = altair_with_screen();
        // MVI A,41H; OUT 11H; MVI A,42H; OUT 11H; HLT
        toggle_in(
            &mut altair,
            0x0000,
            &[0x3E, 0x41, 0xD3, 0x11, 0x3E, 0x42, 0xD3, 0x11, 0x76],
        );

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"AB");
    }

    #[test]
    fn memory_crosses_from_first_card_to_second() {
        let (mut altair, screen) = altair_with_screen();
        toggle_in(&mut altair, 0x7FFF, b"YZ");
        // 0000H: LDA 7FFFH; OUT 11H; LDA 8000H; OUT 11H; HLT
        toggle_in(
            &mut altair,
            0x0000,
            &[
                0x3A, 0xFF, 0x7F, 0xD3, 0x11, 0x3A, 0x00, 0x80, 0xD3, 0x11, 0x76,
            ],
        );

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"YZ");
    }

    #[test]
    fn in_ffh_reads_upper_eight_switches() {
        let (mut altair, screen) = altair_with_screen();
        // 0000H: IN 0FFH; OUT 11H; HLT
        toggle_in(&mut altair, 0x0000, &[0xDB, 0xFF, 0xD3, 0x11, 0x76]);
        altair.front_panel().set_switches(0x4200);

        run_until_halted(&mut altair);

        assert_eq!(*screen.borrow(), b"B"); // 42H
    }

    #[test]
    fn empty_slot_reads_ffh() {
        let mut altair = Altair::builder()
            .plug(Ram::new(0x0000..=0x00FF).unwrap())
            .build();
        // 0000H: LDA 8000H; HLT
        toggle_in(&mut altair, 0x0000, &[0x3A, 0x00, 0x80, 0x76]);

        run_until_halted(&mut altair);

        assert_eq!(altair.cpu.registers.a, 0xFF);
    }
}
