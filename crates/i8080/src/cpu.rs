use crate::instruction::Instruction;
use crate::{MemoryBus, Registers};

/// The Intel 8080 processor.
///
/// `Cpu` holds the processor's state. The machine owns the memory and lends
/// it to the CPU on each call to [`step`](Cpu::step).
///
/// ```
/// use i8080::Cpu;
///
/// let mut memory = [0u8; 0x10000]; // all zeros: NOP everywhere
/// let mut cpu = Cpu::new();
///
/// let cycles = cpu.step(&mut memory);
///
/// assert_eq!(cpu.registers.pc, 1);
/// assert_eq!(cycles, 4);
/// ```
#[derive(Clone, Debug, Default)]
pub struct Cpu {
    /// The registers.
    pub registers: Registers,
}

impl Cpu {
    /// Creates a CPU in its power-on state, ready to run from address 0.
    ///
    /// The real 8080 starts with `PC` at 0 and random values in the other
    /// registers. This emulator sets them all to 0, so two runs of the same
    /// program give the same result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Executes one instruction and returns how many clock cycles it took.
    ///
    /// # Panics
    ///
    /// Panics on opcodes the emulator doesn't support yet.
    pub fn step(&mut self, memory: &mut impl MemoryBus) -> u32 {
        // Fetch and decode: read the instruction's bytes starting at PC.
        let instruction = Instruction::decode(|| self.fetch_byte(memory));

        // Execute. Cycle counts come from the Intel 8080 Microcomputer
        // Systems User's Manual (September 1975):
        // https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf
        match instruction {
            Instruction::Nop => 4,
            Instruction::MviB(value) => {
                self.registers.b = value;
                7
            }
        }
    }

    /// Reads the byte at PC and moves PC to the next one.
    fn fetch_byte(&mut self, memory: &mut impl MemoryBus) -> u8 {
        let byte = memory.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        byte
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Registers holding distinct values, so a test notices when an
    /// instruction writes to the wrong register. PC starts at 0100H.
    fn distinct_registers() -> Registers {
        Registers {
            a: 0x0A,
            b: 0x0B,
            c: 0x0C,
            d: 0x0D,
            e: 0x0E,
            h: 0x01,
            l: 0x02,
            sp: 0xF000,
            pc: 0x0100,
        }
    }

    #[test]
    fn nop_changes_nothing_but_pc() {
        let mut memory = [0u8; 0x10000]; // NOP
        let mut cpu = Cpu::new();
        cpu.registers = distinct_registers();
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0101,
                ..before
            }
        );
        assert_eq!(cycles, 4);
    }

    #[test]
    fn mvi_b_loads_next_byte_into_b() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100..0x0102].copy_from_slice(&[0x06, 0x42]); // MVI B,42H
        let mut cpu = Cpu::new();
        cpu.registers = distinct_registers();
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert_eq!(
            cpu.registers,
            Registers {
                b: 0x42,
                pc: 0x0102,
                ..before
            }
        );
        assert_eq!(cycles, 7);
    }

    #[test]
    fn pc_wraps_from_ffff_to_0000() {
        let mut memory = [0u8; 0x10000]; // NOP
        let mut cpu = Cpu::new();
        cpu.registers.pc = 0xFFFF;

        cpu.step(&mut memory);

        assert_eq!(cpu.registers.pc, 0x0000);
    }

    #[test]
    #[should_panic(expected = "opcode 0x76 is not supported yet")]
    fn unsupported_opcode_panics() {
        let mut memory = [0u8; 0x10000];
        memory[0] = 0x76; // HLT, not implemented yet
        let mut cpu = Cpu::new();

        cpu.step(&mut memory);
    }
}
