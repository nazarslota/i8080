use crate::cycles::{self, HALTED_STEP_CYCLES};
use crate::instruction::{Instruction, Operand};
use crate::{Flags, MemoryBus, Registers};

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
/// assert_eq!(cycles, 4); // NOP takes 4 cycles
/// ```
#[derive(Clone, Debug, Default)]
pub struct Cpu {
    /// The registers.
    pub registers: Registers,
    halted: bool,
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

    /// Returns `true` after a `HLT` instruction stopped the CPU.
    pub const fn is_halted(&self) -> bool {
        self.halted
    }

    /// Executes one instruction and returns how many clock cycles it took.
    ///
    /// A halted CPU executes nothing and returns [`HALTED_STEP_CYCLES`], so
    /// the machine's clock keeps running while the CPU waits.
    ///
    /// # Panics
    ///
    /// Panics on opcodes the emulator doesn't support yet.
    pub fn step(&mut self, memory: &mut impl MemoryBus) -> u32 {
        if self.halted {
            return HALTED_STEP_CYCLES;
        }

        // Fetch and decode: read the instruction's bytes starting at PC.
        let instruction = Instruction::decode(|| self.fetch_byte(memory));

        // Execute: one line per instruction. Instructions with logic get their
        // own method, which returns the cycle count from the `cycles` module.
        match instruction {
            Instruction::Nop => cycles::NOP,
            Instruction::Mvi { dst, value } => self.mvi(memory, dst, value),
            Instruction::Mov { dst, src } => self.mov(memory, dst, src),
            Instruction::Hlt => self.hlt(),
            Instruction::Add { src } => self.add(memory, src),
        }
    }

    /// `MVI dst,value`: loads a constant byte into a register or `M`.
    fn mvi(&mut self, memory: &mut impl MemoryBus, dst: Operand, value: u8) -> u32 {
        self.write_operand(memory, dst, value);
        if dst == Operand::M {
            cycles::MVI_M
        } else {
            cycles::MVI
        }
    }

    /// `MOV dst,src`: copies a byte between registers and `M`.
    fn mov(&mut self, memory: &mut impl MemoryBus, dst: Operand, src: Operand) -> u32 {
        let value = self.read_operand(memory, src);
        self.write_operand(memory, dst, value);
        if dst == Operand::M || src == Operand::M {
            cycles::MOV_M
        } else {
            cycles::MOV
        }
    }

    /// `HLT`: stops the CPU. Later steps execute nothing.
    fn hlt(&mut self) -> u32 {
        self.halted = true;
        cycles::HLT
    }

    /// `ADD src`: adds a register or `M` to `A` and sets every flag.
    fn add(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        let (result, carry) = a.overflowing_add(value);

        self.registers.a = result;
        self.registers.flags = Flags {
            sign: result & 0x80 != 0,
            zero: result == 0,
            aux_carry: (a & 0x0F) + (value & 0x0F) > 0x0F,
            parity: result.count_ones() % 2 == 0,
            carry,
        };

        if src == Operand::M {
            cycles::ADD_M
        } else {
            cycles::ADD
        }
    }

    /// Reads an 8-bit operand. `M` reads memory at the address in `HL`.
    fn read_operand(&self, memory: &mut impl MemoryBus, operand: Operand) -> u8 {
        let r = &self.registers;
        match operand {
            Operand::B => r.b,
            Operand::C => r.c,
            Operand::D => r.d,
            Operand::E => r.e,
            Operand::H => r.h,
            Operand::L => r.l,
            Operand::M => memory.read(r.hl()),
            Operand::A => r.a,
        }
    }

    /// Writes an 8-bit operand. `M` writes to memory at the address in `HL`.
    fn write_operand(&mut self, memory: &mut impl MemoryBus, operand: Operand, value: u8) {
        let r = &mut self.registers;
        match operand {
            Operand::B => r.b = value,
            Operand::C => r.c = value,
            Operand::D => r.d = value,
            Operand::E => r.e = value,
            Operand::H => r.h = value,
            Operand::L => r.l = value,
            Operand::M => memory.write(r.hl(), value),
            Operand::A => r.a = value,
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
            flags: Flags::default(),
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
        assert_eq!(cycles, cycles::NOP);
    }

    #[test]
    fn mvi_loads_next_byte_into_register() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the loaded register holds 42H and PC moved 2 bytes.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0102,
            ..before
        };
        let cases = [
            (0x06, Registers { b: 0x42, ..after }), // MVI B,42H
            (0x0E, Registers { c: 0x42, ..after }), // MVI C,42H
            (0x16, Registers { d: 0x42, ..after }), // MVI D,42H
            (0x1E, Registers { e: 0x42, ..after }), // MVI E,42H
            (0x26, Registers { h: 0x42, ..after }), // MVI H,42H
            (0x2E, Registers { l: 0x42, ..after }), // MVI L,42H
            (0x3E, Registers { a: 0x42, ..after }), // MVI A,42H
        ];

        for (opcode, expected) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100..0x0102].copy_from_slice(&[opcode, 0x42]);
            let mut cpu = Cpu::new();
            cpu.registers = before;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers, expected, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::MVI, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn mvi_m_stores_next_byte_at_hl() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100..0x0102].copy_from_slice(&[0x36, 0x99]); // MVI M,99H
        let mut cpu = Cpu::new();
        cpu.registers = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert_eq!(memory[0x2000], 0x99);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0102,
                ..before
            }
        );
        assert_eq!(cycles, cycles::MVI_M);
    }

    #[test]
    fn mov_copies_register_to_register() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the destination holds the source's value and PC
        // moved 1 byte.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0101,
            ..before
        };
        let cases = [
            (
                0x41,
                Registers {
                    b: before.c,
                    ..after
                },
            ), // MOV B,C
            (
                0x48,
                Registers {
                    c: before.b,
                    ..after
                },
            ), // MOV C,B
            (
                0x5F,
                Registers {
                    e: before.a,
                    ..after
                },
            ), // MOV E,A
            (
                0x62,
                Registers {
                    h: before.d,
                    ..after
                },
            ), // MOV H,D
            (
                0x6B,
                Registers {
                    l: before.e,
                    ..after
                },
            ), // MOV L,E
            (
                0x78,
                Registers {
                    a: before.b,
                    ..after
                },
            ), // MOV A,B
            (0x7F, after), // MOV A,A
        ];

        for (opcode, expected) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = before;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers, expected, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::MOV, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn mov_from_m_reads_memory_at_hl() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100] = 0x7E; // MOV A,M
        memory[0x2000] = 0x99;
        let mut cpu = Cpu::new();
        cpu.registers = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert_eq!(
            cpu.registers,
            Registers {
                a: 0x99,
                pc: 0x0101,
                ..before
            }
        );
        assert_eq!(cycles, cycles::MOV_M);
    }

    #[test]
    fn mov_to_m_writes_memory_at_hl() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100] = 0x77; // MOV M,A
        let mut cpu = Cpu::new();
        cpu.registers = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert_eq!(memory[0x2000], before.a);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0101,
                ..before
            }
        );
        assert_eq!(cycles, cycles::MOV_M);
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
    fn hlt_halts_cpu() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100] = 0x76; // HLT
        let mut cpu = Cpu::new();
        cpu.registers = distinct_registers();
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert!(cpu.is_halted());
        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0101,
                ..before
            }
        );
        assert_eq!(cycles, cycles::HLT);
    }

    #[test]
    fn halted_cpu_executes_nothing() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100..0x0102].copy_from_slice(&[0x06, 0x42]); // MVI B,42H
        let mut cpu = Cpu::new();
        cpu.registers = distinct_registers();
        cpu.halted = true;
        let before = cpu.registers;

        let cycles = cpu.step(&mut memory);

        assert!(cpu.is_halted());
        assert_eq!(cpu.registers, before);
        assert_eq!(cycles, HALTED_STEP_CYCLES);
    }

    #[test]
    fn add_adds_register_to_a() {
        // Opcodes from the Intel manual, each with the sum it should leave
        // in A. A starts at 0AH.
        let cases = [
            (0x80, 0x0A + 0x0B), // ADD B
            (0x81, 0x0A + 0x0C), // ADD C
            (0x82, 0x0A + 0x0D), // ADD D
            (0x83, 0x0A + 0x0E), // ADD E
            (0x84, 0x0A + 0x01), // ADD H
            (0x85, 0x0A + 0x02), // ADD L
            (0x87, 0x0A + 0x0A), // ADD A
        ];

        for (opcode, sum) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, sum, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::ADD, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn add_m_adds_memory_at_hl() {
        let mut memory = [0u8; 0x10000];
        memory[0x0100] = 0x86; // ADD M
        memory[0x2000] = 0x05;
        let mut cpu = Cpu::new();
        cpu.registers = Registers {
            a: 0x10,
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };

        let cycles = cpu.step(&mut memory);

        assert_eq!(cpu.registers.a, 0x15);
        assert_eq!(cycles, cycles::ADD_M);
    }

    #[test]
    fn add_sets_flags_from_result() {
        // Each case: A, B, the sum and the flags it should leave behind. The
        // CPU starts with every flag set, so a flag that should clear does.
        let none = Flags::default();
        let cases = [
            (0x12, 0x34, 0x46, none), // 0100 0110: three 1 bits, odd parity
            (0x70, 0x10, 0x80, Flags { sign: true, ..none }),
            (
                0x0F,
                0x01,
                0x10,
                Flags {
                    aux_carry: true,
                    ..none
                },
            ),
            (
                0x01,
                0x02,
                0x03,
                Flags {
                    parity: true,
                    ..none
                },
            ),
            (
                0x80,
                0x80,
                0x00,
                Flags {
                    zero: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            // The ADD example from the Intel manual: 6CH + 2EH = 9AH.
            (
                0x6C,
                0x2E,
                0x9A,
                Flags {
                    sign: true,
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
        ];

        for (a, b, sum, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0x80; // ADD B
            let mut cpu = Cpu::new();
            cpu.registers = Registers {
                a,
                b,
                flags: Flags {
                    sign: true,
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    carry: true,
                },
                ..distinct_registers()
            };

            cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, sum, "{a:#04X} + {b:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} + {b:#04X}");
        }
    }

    #[test]
    #[should_panic(expected = "opcode 0x88 is not supported yet")]
    fn unsupported_opcode_panics() {
        let mut memory = [0u8; 0x10000];
        memory[0] = 0x88; // ADC B, not implemented yet
        let mut cpu = Cpu::new();

        cpu.step(&mut memory);
    }
}
