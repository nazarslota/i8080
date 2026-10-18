//! The front panel: the switches you operate the Altair with.
//!
//! The Altair has no keyboard and no ROM. You enter a program byte by byte
//! with 16 switches, A0 to A15, and a row of command switches:
//!
//! | Command      | What it does                                           |
//! |--------------|--------------------------------------------------------|
//! | EXAMINE      | sets `PC` to the address on A0 to A15                  |
//! | EXAMINE NEXT | adds 1 to `PC`                                         |
//! | DEPOSIT      | writes the byte on A0 to A7 at `PC`                    |
//! | DEPOSIT NEXT | adds 1 to `PC`, then writes the byte on A0 to A7 there |
//! | RUN          | starts the CPU at `PC`                                 |
//! | STOP         | stops the CPU                                          |
//! | SINGLE STEP  | executes one instruction                               |
//! | RESET        | sets `PC` to 0000H                                     |
//!
//! The panel can't set `PC` itself. For EXAMINE it feeds the CPU a `JMP`
//! through the data bus, followed by the address on the switches, and the
//! CPU executes the jump. For EXAMINE NEXT it feeds a `NOP`, and the CPU
//! moves `PC` past it. DEPOSIT writes to memory without the CPU: the panel
//! puts the switches on the data bus at the address the CPU waits on. RESET
//! pulses the 8080's RESET pin. [Altair 8800 Theory of Operation] describes
//! the circuits.
//!
//! The upper eight switches, A8 to A15, double as the sense switches: a
//! running program reads them with `IN 0FFH`, A15 as bit 7. Altair BASIC and
//! its loaders read them to learn which card carries the tape and the
//! terminal:
//!
//! | Program   | Switches A8 to A15 for the 88-2SIO port A |
//! |-----------|-------------------------------------------|
//! | BASIC 4.x | all down: `00H`                           |
//! | BASIC 3.x | A11 and A10 up: `0CH`                     |
//!
//! The settings come from [Loading Basic with the 88-2SIO Board] by Martin
//! Eberhard (2013), which lists the switches for every card.
//!
//! Each command returns the byte that the data lights now show: the byte at
//! `PC`.
//!
//! [Altair 8800 Theory of Operation]: https://altairclone.com/downloads/manuals/Altair%208800%20Theory%20of%20Operation.pdf
//! [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf

use i8080::{IoBus, MemoryBus};

use super::{Altair, Backplane};

/// The `JMP` opcode, which EXAMINE feeds the CPU.
const JMP_OPCODE: u8 = 0xC3;
/// The `NOP` opcode, which EXAMINE NEXT feeds the CPU.
const NOP_OPCODE: u8 = 0x00;

/// The front panel of an [`Altair`]. Get it with
/// [`Altair::front_panel`].
pub struct FrontPanel<'a> {
    altair: &'a mut Altair,
}

impl<'a> FrontPanel<'a> {
    pub(crate) fn new(altair: &'a mut Altair) -> Self {
        Self { altair }
    }

    /// Sets the 16 switches A0 to A15: bit 0 is A0, and a 1 bit is a switch
    /// up.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.set_switches(0x2000); // A13 up, the rest down
    /// panel.set_switches(0o076); // A7 to A0 in octal groups: 000 111 110
    /// panel.set_switches(0b1010_1010_1010_1010); // A15 down, A14 up, A13 down, ...
    /// ```
    pub fn set_switches(&mut self, switches: u16) {
        self.altair.switches = switches;
    }

    /// EXAMINE: sets `PC` to the address on the switches.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.toggle_in(0x2000, &[0x3E]);
    ///
    /// panel.set_switches(0x2000);
    /// assert_eq!(panel.examine(), 0x3E); // the byte at 2000H
    /// ```
    pub fn examine(&mut self) -> u8 {
        let [low, high] = self.altair.switches.to_le_bytes();
        self.feed(&[JMP_OPCODE, low, high]);
        self.data_lights()
    }

    /// EXAMINE NEXT: moves `PC` to the next address.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.toggle_in(0x2000, &[0x3E, 0x41]);
    /// panel.set_switches(0x2000);
    /// panel.examine();
    ///
    /// assert_eq!(panel.examine_next(), 0x41); // the byte at 2001H
    /// ```
    pub fn examine_next(&mut self) -> u8 {
        self.feed(&[NOP_OPCODE]);
        self.data_lights()
    }

    /// DEPOSIT: writes the byte on switches A0 to A7 at `PC`.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.set_switches(0x2000);
    /// panel.examine();
    /// panel.set_switches(0x003E); // A8 to A15 don't matter here
    ///
    /// assert_eq!(panel.deposit(), 0x3E); // 2000H now holds 3EH
    /// ```
    pub fn deposit(&mut self) -> u8 {
        let [_, byte] = self.altair.switches.to_be_bytes();
        let pc = self.altair.cpu.registers.pc;
        self.altair.backplane.write(pc, byte);
        self.data_lights()
    }

    /// DEPOSIT NEXT: moves `PC` to the next address and writes the byte on
    /// switches A0 to A7 there.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.set_switches(0x2000);
    /// panel.examine();
    /// panel.set_switches(0x003E);
    /// panel.deposit(); // 2000H: 3EH
    /// panel.set_switches(0x0041);
    ///
    /// assert_eq!(panel.deposit_next(), 0x41); // 2001H: 41H
    /// ```
    pub fn deposit_next(&mut self) -> u8 {
        self.examine_next();
        self.deposit()
    }

    /// Toggles in `bytes` from `address` up, the way you would by hand:
    /// EXAMINE the address, DEPOSIT the first byte, DEPOSIT NEXT the rest.
    /// Leaves `PC` at the last byte.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.toggle_in(0x2000, &[0x3E, 0x41, 0x76]); // MVI A,41H; HLT
    ///
    /// panel.set_switches(0x2000);
    /// assert_eq!(panel.examine(), 0x3E);
    /// assert_eq!(panel.examine_next(), 0x41);
    /// assert_eq!(panel.examine_next(), 0x76);
    /// ```
    pub fn toggle_in(&mut self, address: u16, bytes: &[u8]) {
        self.set_switches(address);
        self.examine();
        for (index, &byte) in bytes.iter().enumerate() {
            self.set_switches(u16::from(byte));
            if index == 0 {
                self.deposit();
            } else {
                self.deposit_next();
            }
        }
    }

    /// RUN: lets the CPU run from `PC`.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.run();
    ///
    /// assert!(altair.is_running());
    /// ```
    pub fn run(&mut self) {
        self.altair.running = true;
    }

    /// STOP: stops the CPU.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.run();
    /// panel.stop();
    ///
    /// assert!(!altair.is_running());
    /// ```
    pub fn stop(&mut self) {
        self.altair.running = false;
    }

    /// SINGLE STEP: executes one instruction and returns its clock cycles.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.toggle_in(0x0000, &[0x3E, 0x41, 0x76]); // MVI A,41H; HLT
    /// panel.set_switches(0x0000);
    /// panel.examine();
    ///
    /// assert_eq!(panel.single_step(), 7); // MVI A,41H
    /// assert_eq!(panel.single_step(), 7); // HLT
    /// assert!(altair.is_halted());
    /// ```
    pub fn single_step(&mut self) -> u32 {
        self.altair.step()
    }

    /// RESET: sets `PC` to 0000H, turns interrupts off and wakes a halted
    /// CPU.
    ///
    /// ```
    /// # use altair::{Altair, cards::{Cpu, Ram}};
    /// # let mut altair = Altair::builder().plug(Cpu::new()).plug(Ram::new(0x0000..=0xFFFF)).build();
    /// let mut panel = altair.front_panel();
    /// panel.toggle_in(0x0000, &[0x76]); // HLT
    /// panel.set_switches(0x2000);
    /// panel.examine(); // PC at 2000H
    ///
    /// panel.reset(); // PC at 0000H
    ///
    /// panel.single_step();
    /// assert!(altair.is_halted());
    /// ```
    pub fn reset(&mut self) {
        self.altair.cpu.reset();
    }

    /// Feeds `instruction` to the CPU through the data bus and lets the CPU
    /// execute it, as the panel circuits do.
    fn feed(&mut self, instruction: &[u8]) {
        let mut bus = FedBus {
            instruction: instruction.iter(),
            backplane: &mut self.altair.backplane,
        };
        self.altair.cpu.step(&mut bus);
    }

    /// Returns the byte at `PC`, the one the data lights show.
    fn data_lights(&mut self) -> u8 {
        let pc = self.altair.cpu.registers.pc;
        self.altair.backplane.read(pc)
    }
}

/// The bus while the panel feeds the CPU an instruction: memory reads get
/// the instruction's bytes, everything else goes to the backplane.
struct FedBus<'a> {
    instruction: core::slice::Iter<'a, u8>,
    backplane: &'a mut Backplane,
}

impl MemoryBus for FedBus<'_> {
    fn read(&mut self, addr: u16) -> u8 {
        match self.instruction.next() {
            Some(&byte) => byte,
            None => self.backplane.read(addr),
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.backplane.write(addr, value);
    }
}

impl IoBus for FedBus<'_> {
    fn input(&mut self, port: u8) -> u8 {
        self.backplane.input(port)
    }

    fn output(&mut self, port: u8, value: u8) {
        self.backplane.output(port, value);
    }
}

#[cfg(test)]
mod tests {
    use crate::Altair;
    use crate::cards::{Cpu, Ram};

    fn altair() -> Altair {
        Altair::builder()
            .plug(Cpu::new())
            .plug(Ram::new(0x0000..=0xFFFF))
            .build()
    }

    #[test]
    fn examine_sets_pc_to_switches() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.set_switches(0x1234);

        panel.examine();

        assert_eq!(altair.cpu.registers.pc, 0x1234);
    }

    #[test]
    fn deposit_writes_low_byte_of_switches_at_pc() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.set_switches(0x2000);
        panel.examine();
        panel.set_switches(0xFF99);

        let lights = panel.deposit();

        assert_eq!(lights, 0x99);
        assert_eq!(panel.examine(), 0x00); // the switches still say FF99H
        panel.set_switches(0x2000);
        assert_eq!(panel.examine(), 0x99);
    }

    #[test]
    fn deposit_next_writes_at_next_address() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.set_switches(0x2000);
        panel.examine();
        panel.set_switches(0x0011);
        panel.deposit();
        panel.set_switches(0x0022);

        panel.deposit_next();

        panel.set_switches(0x2000);
        assert_eq!(panel.examine(), 0x11);
        assert_eq!(panel.examine_next(), 0x22);
        assert_eq!(altair.cpu.registers.pc, 0x2001);
    }

    #[test]
    fn toggle_in_deposits_bytes_from_address() {
        let mut altair = altair();
        let mut panel = altair.front_panel();

        panel.toggle_in(0x2000, &[0x11, 0x22, 0x33]);

        panel.set_switches(0x2000);
        assert_eq!(panel.examine(), 0x11);
        assert_eq!(panel.examine_next(), 0x22);
        assert_eq!(panel.examine_next(), 0x33);
    }

    #[test]
    fn run_and_stop_switch_running() {
        let mut altair = altair();

        altair.front_panel().run();
        let running = altair.is_running();
        altair.front_panel().stop();

        assert!(running);
        assert!(!altair.is_running());
    }

    #[test]
    fn single_step_executes_one_instruction() {
        let mut altair = altair(); // all zeros: NOP everywhere

        let cycles = altair.front_panel().single_step();

        assert_eq!(cycles, 4);
        assert_eq!(altair.cpu.registers.pc, 0x0001);
    }

    #[test]
    fn reset_wakes_halted_cpu() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.toggle_in(0x0000, &[0x76]); // HLT
        panel.set_switches(0x0000);
        panel.examine();
        panel.single_step();

        panel.reset();

        assert!(!altair.is_halted());
    }

    #[test]
    fn reset_sets_pc_to_0000h() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.set_switches(0x1234);
        panel.examine();

        panel.reset();

        assert_eq!(altair.cpu.registers.pc, 0x0000);
    }
}
