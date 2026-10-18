use i8080::{IoBus, MemoryBus};

mod backplane;
mod front_panel;

pub use backplane::Backplane;
pub use front_panel::FrontPanel;

use crate::cards::{Card, Cpu};

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
/// use altair::cards::{Cpu, Ram, Sio2, Terminal};
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
///     .plug(Cpu::new())
///     .plug(Ram::new(0x0000..=0xFFFF))
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
    /// Starts an Altair with empty slots.
    pub fn builder() -> AltairBuilder {
        AltairBuilder {
            cpu: None,
            backplane: Backplane::new(),
        }
    }

    /// Returns the front panel, the switches you operate the machine with.
    pub fn front_panel(&mut self) -> FrontPanel<'_> {
        FrontPanel::new(self)
    }

    /// Executes one instruction and returns how many clock cycles it took.
    ///
    /// `step` executes whether the front panel says RUN or STOP: you decide
    /// when to call it. Check [`is_running`](Altair::is_running) first to
    /// follow the panel.
    pub fn step(&mut self) -> u32 {
        let [sense_switches, _] = self.switches.to_be_bytes();
        let mut bus = MachineBus {
            backplane: &mut self.backplane,
            sense_switches,
        };
        self.cpu.step(&mut bus)
    }

    /// Returns `true` after RUN on the front panel, `false` after STOP.
    pub const fn is_running(&self) -> bool {
        self.running
    }

    /// Returns `true` after a `HLT` instruction stopped the CPU.
    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }
}

/// Collects the cards for an [`Altair`], slot by slot.
pub struct AltairBuilder {
    cpu: Option<Cpu>,
    backplane: Backplane,
}

impl AltairBuilder {
    /// Plugs `card` into the next free slot.
    ///
    /// # Panics
    ///
    /// Panics if you plug in a second CPU card.
    pub fn plug(mut self, card: impl Plug) -> Self {
        card.plug_into(&mut self);
        self
    }

    /// Builds the machine: every switch down, the CPU stopped, `PC` at 0000H.
    ///
    /// # Panics
    ///
    /// Panics if you plugged in no CPU card.
    pub fn build(self) -> Altair {
        Altair {
            cpu: self
                .cpu
                .expect("plug in a CPU card: .plug(cards::Cpu::new())"),
            backplane: self.backplane,
            switches: 0x0000,
            running: false,
        }
    }
}

/// A card that [`AltairBuilder::plug`] takes: the CPU card or any [`Card`].
///
/// The CPU card drives the bus and the other cards answer it, so the machine
/// keeps them apart.
pub trait Plug {
    /// Puts the card into the machine that `builder` assembles.
    fn plug_into(self, builder: &mut AltairBuilder);
}

impl Plug for Cpu {
    fn plug_into(self, builder: &mut AltairBuilder) {
        assert!(builder.cpu.is_none(), "the Altair takes one CPU card");
        builder.cpu = Some(self);
    }
}

impl<T: Card + 'static> Plug for T {
    fn plug_into(self, builder: &mut AltairBuilder) {
        builder.backplane.plug(self);
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
    use crate::cards::{Cpu, Ram, Sio2, Terminal};

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
            .plug(Cpu::new())
            .plug(Ram::new(0x0000..=0x7FFF))
            .plug(Ram::new(0x8000..=0xFFFF))
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
            .plug(Cpu::new())
            .plug(Ram::new(0x0000..=0x00FF))
            .build();
        // 0000H: LDA 8000H; HLT
        toggle_in(&mut altair, 0x0000, &[0x3A, 0x00, 0x80, 0x76]);

        run_until_halted(&mut altair);

        assert_eq!(altair.cpu.registers.a, 0xFF);
    }

    #[test]
    #[should_panic(expected = "plug in a CPU card")]
    fn build_without_cpu_card_panics() {
        Altair::builder().plug(Ram::new(0x0000..=0xFFFF)).build();
    }

    #[test]
    #[should_panic(expected = "one CPU card")]
    fn second_cpu_card_panics() {
        Altair::builder().plug(Cpu::new()).plug(Cpu::new());
    }
}
