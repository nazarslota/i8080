use crate::alu;
use crate::cycles::{self, HALTED_STEP_CYCLES};
use crate::instruction::{Instruction, Operand};
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
            Instruction::Adc { src } => self.adc(memory, src),
            Instruction::Sub { src } => self.sub(memory, src),
            Instruction::Sbb { src } => self.sbb(memory, src),
            Instruction::Ana { src } => self.ana(memory, src),
            Instruction::Xra { src } => self.xra(memory, src),
            Instruction::Ora { src } => self.ora(memory, src),
            Instruction::Cmp { src } => self.cmp(memory, src),
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

    /// `ADD src`: adds a register or `M` to `A`.
    fn add(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::add(a, value, false);
        if src == Operand::M {
            cycles::ADD_M
        } else {
            cycles::ADD
        }
    }

    /// `ADC src`: adds a register or `M` and the carry flag to `A`.
    fn adc(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::add(a, value, self.registers.flags.carry);
        if src == Operand::M {
            cycles::ADC_M
        } else {
            cycles::ADC
        }
    }

    /// `SUB src`: subtracts a register or `M` from `A`.
    fn sub(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::subtract(a, value, false);
        if src == Operand::M {
            cycles::SUB_M
        } else {
            cycles::SUB
        }
    }

    /// `SBB src`: subtracts a register or `M` and the carry flag from `A`.
    fn sbb(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) =
            alu::subtract(a, value, self.registers.flags.carry);
        if src == Operand::M {
            cycles::SBB_M
        } else {
            cycles::SBB
        }
    }

    /// `ANA src`: `A = A AND src`, with a register or `M` as `src`.
    fn ana(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::and(a, value);
        if src == Operand::M {
            cycles::ANA_M
        } else {
            cycles::ANA
        }
    }

    /// `XRA src`: `A = A XOR src`, with a register or `M` as `src`.
    fn xra(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::xor(a, value);
        if src == Operand::M {
            cycles::XRA_M
        } else {
            cycles::XRA
        }
    }

    /// `ORA src`: `A = A OR src`, with a register or `M` as `src`.
    fn ora(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::or(a, value);
        if src == Operand::M {
            cycles::ORA_M
        } else {
            cycles::ORA
        }
    }

    /// `CMP src`: compares `A` with a register or `M`: sets the flags
    /// from `A - src` and leaves `A` as it was.
    fn cmp(&mut self, memory: &mut impl MemoryBus, src: Operand) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (_, self.registers.flags) = alu::subtract(a, value, false);
        if src == Operand::M {
            cycles::CMP_M
        } else {
            cycles::CMP
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
    use crate::Flags;

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
    fn adc_adds_register_and_carry_to_a() {
        // Opcodes from the Intel manual, each with the carry flag before the
        // step and the sum it should leave in A. A starts at 0AH.
        let cases = [
            (0x88, false, 0x0A + 0x0B),    // ADC B
            (0x89, true, 0x0A + 0x0C + 1), // ADC C
            (0x8A, true, 0x0A + 0x0D + 1), // ADC D
            (0x8B, false, 0x0A + 0x0E),    // ADC E
            (0x8C, true, 0x0A + 0x01 + 1), // ADC H
            (0x8D, false, 0x0A + 0x02),    // ADC L
            (0x8F, true, 0x0A + 0x0A + 1), // ADC A
        ];

        for (opcode, carry, sum) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();
            cpu.registers.flags.carry = carry;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, sum, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::ADC, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn adc_sets_flags_from_result() {
        // Each case: A, B and the carry before the step, then the sum and
        // the flags it should leave behind.
        let none = Flags::default();
        let cases = [
            // The carry pushes the low 4 bits past 0FH.
            (
                0x0F,
                0x00,
                true,
                0x10,
                Flags {
                    aux_carry: true,
                    ..none
                },
            ),
            // The carry pushes the sum past FFH.
            (
                0xFF,
                0x00,
                true,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            // The ADC examples from the Intel manual: 42H + 3DH, first with
            // the carry at 0, then at 1.
            (0x42, 0x3D, false, 0x7F, none),
            (
                0x42,
                0x3D,
                true,
                0x80,
                Flags {
                    sign: true,
                    aux_carry: true,
                    ..none
                },
            ),
        ];

        for (a, b, carry, sum, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0x88; // ADC B
            let mut cpu = Cpu::new();
            cpu.registers = Registers {
                a,
                b,
                flags: Flags { carry, ..none },
                ..distinct_registers()
            };

            cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, sum, "{a:#04X} + {b:#04X} + {carry}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} + {b:#04X} + {carry}");
        }
    }

    #[test]
    fn sub_subtracts_register_from_a() {
        // Opcodes from the Intel manual, each with the difference it should
        // leave in A. A starts at 0AH, so some differences wrap below 00H.
        let cases = [
            (0x90, 0xFF), // SUB B: 0AH - 0BH
            (0x91, 0xFE), // SUB C: 0AH - 0CH
            (0x92, 0xFD), // SUB D: 0AH - 0DH
            (0x93, 0xFC), // SUB E: 0AH - 0EH
            (0x94, 0x09), // SUB H: 0AH - 01H
            (0x95, 0x08), // SUB L: 0AH - 02H
            (0x97, 0x00), // SUB A: 0AH - 0AH
        ];

        for (opcode, difference) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, difference, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::SUB, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn sub_sets_flags_from_result() {
        // Each case: A, B, the difference and the flags it should leave
        // behind. The CPU starts with every flag set, so a flag that should
        // clear does.
        let none = Flags::default();
        let cases = [
            // The SUB example from the Intel manual: 3EH - 3EH = 00H.
            (
                0x3E,
                0x3E,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            // 05H - 06H borrows: the result wraps to FFH and CY is 1.
            (
                0x05,
                0x06,
                0xFF,
                Flags {
                    sign: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            // 10H - 01H borrows from the high 4 bits, so AC is 0.
            (
                0x10,
                0x01,
                0x0F,
                Flags {
                    parity: true,
                    ..none
                },
            ),
            // 12H - 01H needs no borrow from the high 4 bits, so AC is 1.
            (
                0x12,
                0x01,
                0x11,
                Flags {
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
        ];

        for (a, b, difference, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0x90; // SUB B
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

            assert_eq!(cpu.registers.a, difference, "{a:#04X} - {b:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} - {b:#04X}");
        }
    }

    #[test]
    fn sbb_subtracts_register_and_carry_from_a() {
        // Opcodes from the Intel manual, each with the carry flag before the
        // step and the difference it should leave in A. A starts at 0AH.
        let cases = [
            (0x98, true, 0xFE),  // SBB B: 0AH - 0BH - 1
            (0x99, false, 0xFE), // SBB C: 0AH - 0CH
            (0x9A, true, 0xFC),  // SBB D: 0AH - 0DH - 1
            (0x9B, false, 0xFC), // SBB E: 0AH - 0EH
            (0x9C, true, 0x08),  // SBB H: 0AH - 01H - 1
            (0x9D, false, 0x08), // SBB L: 0AH - 02H
            (0x9F, true, 0xFF),  // SBB A: 0AH - 0AH - 1
        ];

        for (opcode, carry, difference) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();
            cpu.registers.flags.carry = carry;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, difference, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::SBB, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn sbb_sets_flags_from_result() {
        // Each case: A, L and the carry before the step, then the difference
        // and the flags it should leave behind.
        let none = Flags::default();
        let cases = [
            // The SBB example from the Intel manual: 04H - 02H - 1 = 01H.
            (
                0x04,
                0x02,
                true,
                0x01,
                Flags {
                    aux_carry: true,
                    ..none
                },
            ),
            // The carry alone makes 00H - 00H borrow.
            (
                0x00,
                0x00,
                true,
                0xFF,
                Flags {
                    sign: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
        ];

        for (a, l, carry, difference, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0x9D; // SBB L
            let mut cpu = Cpu::new();
            cpu.registers = Registers {
                a,
                l,
                flags: Flags { carry, ..none },
                ..distinct_registers()
            };

            cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, difference, "{a:#04X} - {l:#04X} - {carry}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} - {l:#04X} - {carry}");
        }
    }

    #[test]
    fn alu_with_m_reads_memory_at_hl() {
        // Each case: an opcode with M as the source, the value it should
        // leave in A and its cycle count. A starts at 10H, memory at HL
        // holds 05H and the carry flag is 1.
        let cases = [
            (0x8E, 0x16, cycles::ADC_M), // ADC M: 10H + 05H + 1
            (0x96, 0x0B, cycles::SUB_M), // SUB M: 10H - 05H
            (0x9E, 0x0A, cycles::SBB_M), // SBB M: 10H - 05H - 1
            (0xA6, 0x00, cycles::ANA_M), // ANA M: 10H AND 05H
            (0xAE, 0x15, cycles::XRA_M), // XRA M: 10H XOR 05H
            (0xB6, 0x15, cycles::ORA_M), // ORA M: 10H OR 05H
            (0xBE, 0x10, cycles::CMP_M), // CMP M: A stays 10H
        ];

        for (opcode, result, expected_cycles) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            memory[0x2000] = 0x05;
            let mut cpu = Cpu::new();
            cpu.registers = Registers {
                a: 0x10,
                h: 0x20,
                l: 0x00,
                ..distinct_registers()
            };
            cpu.registers.flags.carry = true;

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, result, "opcode {opcode:#04X}");
            assert_eq!(cycles, expected_cycles, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn ana_combines_register_with_a() {
        // Opcodes from the Intel manual, each with the value it should leave
        // in A. A starts at 0AH.
        let cases = [
            (0xA0, 0x0A & 0x0B), // ANA B
            (0xA1, 0x0A & 0x0C), // ANA C
            (0xA2, 0x0A & 0x0D), // ANA D
            (0xA3, 0x0A & 0x0E), // ANA E
            (0xA4, 0x0A & 0x01), // ANA H
            (0xA5, 0x0A & 0x02), // ANA L
            (0xA7, 0x0A & 0x0A), // ANA A
        ];

        for (opcode, result) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, result, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::ANA, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn ana_sets_flags_from_result() {
        // Each case: A, B, the result and the flags it should leave behind.
        // The CPU starts with every flag set, so a flag that should clear
        // does.
        let none = Flags::default();
        let cases = [
            // The ANA example from the Intel manual: FCH AND 0FH = 0CH.
            (
                0xFC,
                0x0F,
                0x0C,
                Flags {
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            // Bit 3 is 0 in both operands, so AC is 0.
            (0x81, 0x80, 0x80, Flags { sign: true, ..none }),
            (
                0xF0,
                0x0F,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
        ];

        for (a, b, result, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0xA0; // ANA B
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

            assert_eq!(cpu.registers.a, result, "{a:#04X} ANA {b:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} ANA {b:#04X}");
        }
    }

    #[test]
    fn xra_combines_register_with_a() {
        // Opcodes from the Intel manual, each with the value it should leave
        // in A. A starts at 0AH.
        let cases = [
            (0xA8, 0x0A ^ 0x0B), // XRA B
            (0xA9, 0x0A ^ 0x0C), // XRA C
            (0xAA, 0x0A ^ 0x0D), // XRA D
            (0xAB, 0x0A ^ 0x0E), // XRA E
            (0xAC, 0x0A ^ 0x01), // XRA H
            (0xAD, 0x0A ^ 0x02), // XRA L
            (0xAF, 0x0A ^ 0x0A), // XRA A
        ];

        for (opcode, result) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, result, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::XRA, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn xra_sets_flags_from_result() {
        // Each case: A, B, the result and the flags it should leave behind.
        // The CPU starts with every flag set, so a flag that should clear
        // does.
        let none = Flags::default();
        let cases = [
            // A value XOR itself is 0.
            (
                0x5C,
                0x5C,
                0x00,
                Flags {
                    zero: true,
                    parity: true,
                    ..none
                },
            ),
            // XOR with FFH inverts every bit.
            (
                0x5C,
                0xFF,
                0xA3,
                Flags {
                    sign: true,
                    parity: true,
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
        ];

        for (a, b, result, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0xA8; // XRA B
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

            assert_eq!(cpu.registers.a, result, "{a:#04X} XRA {b:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} XRA {b:#04X}");
        }
    }

    #[test]
    fn ora_combines_register_with_a() {
        // Opcodes from the Intel manual, each with the value it should leave
        // in A. A starts at 0AH.
        let cases = [
            (0xB0, 0x0A | 0x0B), // ORA B
            (0xB1, 0x0A | 0x0C), // ORA C
            (0xB2, 0x0A | 0x0D), // ORA D
            (0xB3, 0x0A | 0x0E), // ORA E
            (0xB4, 0x0A | 0x01), // ORA H
            (0xB5, 0x0A | 0x02), // ORA L
            (0xB7, 0x0A | 0x0A), // ORA A
        ];

        for (opcode, result) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, result, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::ORA, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn ora_sets_flags_from_result() {
        // Each case: A, B, the result and the flags it should leave behind.
        // The CPU starts with every flag set, so a flag that should clear
        // does.
        let none = Flags::default();
        let cases = [
            // The ORA example from the Intel manual: 33H OR 0FH = 3FH.
            (
                0x33,
                0x0F,
                0x3F,
                Flags {
                    parity: true,
                    ..none
                },
            ),
            (
                0x80,
                0x01,
                0x81,
                Flags {
                    sign: true,
                    parity: true,
                    ..none
                },
            ),
            (
                0x00,
                0x00,
                0x00,
                Flags {
                    zero: true,
                    parity: true,
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
        ];

        for (a, b, result, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0xB0; // ORA B
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

            assert_eq!(cpu.registers.a, result, "{a:#04X} ORA {b:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} ORA {b:#04X}");
        }
    }

    #[test]
    fn cmp_keeps_a_and_compares_register() {
        // Opcodes from the Intel manual, each with the Z and CY flags it
        // should leave behind. A starts at 0AH: Z = 1 means equal, CY = 1
        // means the register is larger.
        let cases = [
            (0xB8, false, true),  // CMP B: 0AH < 0BH
            (0xB9, false, true),  // CMP C: 0AH < 0CH
            (0xBA, false, true),  // CMP D: 0AH < 0DH
            (0xBB, false, true),  // CMP E: 0AH < 0EH
            (0xBC, false, false), // CMP H: 0AH > 01H
            (0xBD, false, false), // CMP L: 0AH > 02H
            (0xBF, true, false),  // CMP A: 0AH = 0AH
        ];

        for (opcode, zero, carry) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = opcode;
            let mut cpu = Cpu::new();
            cpu.registers = distinct_registers();

            let cycles = cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, 0x0A, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.flags.zero, zero, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.flags.carry, carry, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::CMP, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn cmp_sets_flags_like_sub() {
        // The CMP E examples from the Intel manual. Each case: A, E and the
        // flags CMP should leave behind. CMP compares unsigned values, so
        // E5H counts as larger than 05H.
        let none = Flags::default();
        let cases = [
            (
                0x0A,
                0x05,
                Flags {
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            (
                0x02,
                0x05,
                Flags {
                    sign: true,
                    carry: true,
                    ..none
                },
            ),
            (
                0xE5,
                0x05,
                Flags {
                    sign: true,
                    aux_carry: true,
                    ..none
                },
            ),
        ];

        for (a, e, flags) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0100] = 0xBB; // CMP E
            let mut cpu = Cpu::new();
            cpu.registers = Registers {
                a,
                e,
                ..distinct_registers()
            };

            cpu.step(&mut memory);

            assert_eq!(cpu.registers.a, a, "{a:#04X} CMP {e:#04X}");
            assert_eq!(cpu.registers.flags, flags, "{a:#04X} CMP {e:#04X}");
        }
    }

    #[test]
    #[should_panic(expected = "opcode 0x01 is not supported yet")]
    fn unsupported_opcode_panics() {
        let mut memory = [0u8; 0x10000];
        memory[0] = 0x01; // LXI B,nn, not implemented yet
        let mut cpu = Cpu::new();

        cpu.step(&mut memory);
    }
}
