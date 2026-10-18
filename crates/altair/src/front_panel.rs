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
//! [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf

use i8080::MemoryBus;

use crate::Altair;

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
    pub fn set_switches(&mut self, switches: u16) {
        self.altair.switches = switches;
    }

    /// EXAMINE: sets `PC` to the address on the switches.
    pub fn examine(&mut self) -> u8 {
        self.altair.cpu.registers.pc = self.altair.switches;
        self.data_lights()
    }

    /// EXAMINE NEXT: moves `PC` to the next address.
    pub fn examine_next(&mut self) -> u8 {
        let registers = &mut self.altair.cpu.registers;
        registers.pc = registers.pc.wrapping_add(1);
        self.data_lights()
    }

    /// DEPOSIT: writes the byte on switches A0 to A7 at `PC`.
    pub fn deposit(&mut self) -> u8 {
        let [_, byte] = self.altair.switches.to_be_bytes();
        let pc = self.altair.cpu.registers.pc;
        self.altair.backplane.write(pc, byte);
        self.data_lights()
    }

    /// DEPOSIT NEXT: moves `PC` to the next address and writes the byte on
    /// switches A0 to A7 there.
    pub fn deposit_next(&mut self) -> u8 {
        self.examine_next();
        self.deposit()
    }

    /// Toggles in `bytes` from `address` up, the way you would by hand:
    /// EXAMINE the address, DEPOSIT the first byte, DEPOSIT NEXT the rest.
    /// Leaves `PC` at the last byte.
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
    pub fn run(&mut self) {
        self.altair.running = true;
    }

    /// STOP: stops the CPU.
    pub fn stop(&mut self) {
        self.altair.running = false;
    }

    /// SINGLE STEP: executes one instruction and returns its clock cycles.
    pub fn single_step(&mut self) -> u32 {
        self.altair.step()
    }

    /// RESET: sets `PC` to 0000H.
    pub fn reset(&mut self) {
        self.altair.cpu.registers.pc = 0x0000;
    }

    /// Returns the byte at `PC`, the one the data lights show.
    fn data_lights(&mut self) -> u8 {
        let pc = self.altair.cpu.registers.pc;
        self.altair.backplane.read(pc)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Altair, Ram};

    fn altair() -> Altair {
        Altair::builder().plug(Ram::new(0x0000..=0xFFFF)).build()
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
    fn reset_sets_pc_to_0000h() {
        let mut altair = altair();
        let mut panel = altair.front_panel();
        panel.set_switches(0x1234);
        panel.examine();

        panel.reset();

        assert_eq!(altair.cpu.registers.pc, 0x0000);
    }
}
