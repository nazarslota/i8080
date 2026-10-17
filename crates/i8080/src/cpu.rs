use crate::alu;
use crate::cycles::{self, HALTED_STEP_CYCLES};
use crate::instruction::{Instruction, Operand8, Operand16};
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
            Instruction::Lxi { dst, value } => self.lxi(dst, value),
            Instruction::Dad { src } => self.dad(src),
            Instruction::Stax { dst } => self.stax(memory, dst),
            Instruction::Ldax { src } => self.ldax(memory, src),
            Instruction::Shld { address } => self.shld(memory, address),
            Instruction::Lhld { address } => self.lhld(memory, address),
            Instruction::Sta { address } => self.sta(memory, address),
            Instruction::Lda { address } => self.lda(memory, address),
            Instruction::Inx { pair } => self.inx(pair),
            Instruction::Dcx { pair } => self.dcx(pair),
            Instruction::Inr { dst } => self.inr(memory, dst),
            Instruction::Dcr { dst } => self.dcr(memory, dst),
            Instruction::Mvi { dst, value } => self.mvi(memory, dst, value),
            Instruction::Rlc => self.rlc(),
            Instruction::Rrc => self.rrc(),
            Instruction::Ral => self.ral(),
            Instruction::Rar => self.rar(),
            Instruction::Daa => self.daa(),
            Instruction::Cma => self.cma(),
            Instruction::Stc => self.stc(),
            Instruction::Cmc => self.cmc(),
            Instruction::Hlt => self.hlt(),
            Instruction::Mov { dst, src } => self.mov(memory, dst, src),
            Instruction::Add { src } => self.add(memory, src),
            Instruction::Adc { src } => self.adc(memory, src),
            Instruction::Sub { src } => self.sub(memory, src),
            Instruction::Sbb { src } => self.sbb(memory, src),
            Instruction::Ana { src } => self.ana(memory, src),
            Instruction::Xra { src } => self.xra(memory, src),
            Instruction::Ora { src } => self.ora(memory, src),
            Instruction::Cmp { src } => self.cmp(memory, src),
            Instruction::Xchg => self.xchg(),
            Instruction::Adi { value } => self.adi(value),
            Instruction::Aci { value } => self.aci(value),
            Instruction::Sui { value } => self.sui(value),
            Instruction::Sbi { value } => self.sbi(value),
            Instruction::Ani { value } => self.ani(value),
            Instruction::Xri { value } => self.xri(value),
            Instruction::Ori { value } => self.ori(value),
            Instruction::Cpi { value } => self.cpi(value),
        }
    }

    /// `LXI dst,value`: loads a 16-bit constant into a register pair.
    fn lxi(&mut self, dst: Operand16, value: u16) -> u32 {
        self.write_pair(dst, value);
        cycles::LXI
    }

    /// `DAD src`: adds a register pair to `HL`. Sets `CY` and no other flag.
    fn dad(&mut self, src: Operand16) -> u32 {
        let value = self.read_pair(src);
        let (sum, carry) = self.registers.hl().overflowing_add(value);
        self.registers.set_hl(sum);
        self.registers.flags.carry = carry;
        cycles::DAD
    }

    /// `STAX dst`: stores `A` at the address in `BC` or `DE`.
    fn stax(&mut self, memory: &mut impl MemoryBus, dst: Operand16) -> u32 {
        memory.write(self.read_pair(dst), self.registers.a);
        cycles::STAX
    }

    /// `LDAX src`: loads `A` from the address in `BC` or `DE`.
    fn ldax(&mut self, memory: &mut impl MemoryBus, src: Operand16) -> u32 {
        self.registers.a = memory.read(self.read_pair(src));
        cycles::LDAX
    }

    /// `SHLD address`: stores `L` at `address` and `H` at the next byte.
    fn shld(&mut self, memory: &mut impl MemoryBus, address: u16) -> u32 {
        memory.write(address, self.registers.l);
        memory.write(address.wrapping_add(1), self.registers.h);
        cycles::SHLD
    }

    /// `LHLD address`: loads `L` from `address` and `H` from the next byte.
    fn lhld(&mut self, memory: &mut impl MemoryBus, address: u16) -> u32 {
        self.registers.l = memory.read(address);
        self.registers.h = memory.read(address.wrapping_add(1));
        cycles::LHLD
    }

    /// `STA address`: stores `A` at `address`.
    fn sta(&mut self, memory: &mut impl MemoryBus, address: u16) -> u32 {
        memory.write(address, self.registers.a);
        cycles::STA
    }

    /// `LDA address`: loads `A` from `address`.
    fn lda(&mut self, memory: &mut impl MemoryBus, address: u16) -> u32 {
        self.registers.a = memory.read(address);
        cycles::LDA
    }

    /// `INX pair`: adds 1 to a register pair. Sets no flags.
    fn inx(&mut self, pair: Operand16) -> u32 {
        let value = self.read_pair(pair);
        self.write_pair(pair, value.wrapping_add(1));
        cycles::INX
    }

    /// `DCX pair`: subtracts 1 from a register pair. Sets no flags.
    fn dcx(&mut self, pair: Operand16) -> u32 {
        let value = self.read_pair(pair);
        self.write_pair(pair, value.wrapping_sub(1));
        cycles::DCX
    }

    /// `INR dst`: adds 1 to a register or `M`. Sets every flag but `CY`.
    fn inr(&mut self, memory: &mut impl MemoryBus, dst: Operand8) -> u32 {
        let value = self.read_operand(memory, dst);
        let (result, flags) = alu::add(value, 1, false);
        self.write_operand(memory, dst, result);
        self.registers.flags = Flags {
            carry: self.registers.flags.carry,
            ..flags
        };
        if dst == Operand8::M {
            cycles::INR_M
        } else {
            cycles::INR
        }
    }

    /// `DCR dst`: subtracts 1 from a register or `M`. Sets every flag but
    /// `CY`.
    fn dcr(&mut self, memory: &mut impl MemoryBus, dst: Operand8) -> u32 {
        let value = self.read_operand(memory, dst);
        let (result, flags) = alu::subtract(value, 1, false);
        self.write_operand(memory, dst, result);
        self.registers.flags = Flags {
            carry: self.registers.flags.carry,
            ..flags
        };
        if dst == Operand8::M {
            cycles::DCR_M
        } else {
            cycles::DCR
        }
    }

    /// `MVI dst,value`: loads a constant byte into a register or `M`.
    fn mvi(&mut self, memory: &mut impl MemoryBus, dst: Operand8, value: u8) -> u32 {
        self.write_operand(memory, dst, value);
        if dst == Operand8::M {
            cycles::MVI_M
        } else {
            cycles::MVI
        }
    }

    /// `RLC`: rotates `A` left. Bit 7 goes to bit 0 and to `CY`.
    fn rlc(&mut self) -> u32 {
        let a = self.registers.a;
        self.registers.a = a.rotate_left(1);
        self.registers.flags.carry = a & 0x80 != 0;
        cycles::RLC
    }

    /// `RRC`: rotates `A` right. Bit 0 goes to bit 7 and to `CY`.
    fn rrc(&mut self) -> u32 {
        let a = self.registers.a;
        self.registers.a = a.rotate_right(1);
        self.registers.flags.carry = a & 0x01 != 0;
        cycles::RRC
    }

    /// `RAL`: rotates `A` left through `CY`. Bit 7 goes to `CY`, and the
    /// old `CY` goes to bit 0.
    fn ral(&mut self) -> u32 {
        let a = self.registers.a;
        let carry = self.registers.flags.carry;
        self.registers.a = (a << 1) | u8::from(carry);
        self.registers.flags.carry = a & 0x80 != 0;
        cycles::RAL
    }

    /// `RAR`: rotates `A` right through `CY`. Bit 0 goes to `CY`, and the
    /// old `CY` goes to bit 7.
    fn rar(&mut self) -> u32 {
        let a = self.registers.a;
        let carry = self.registers.flags.carry;
        self.registers.a = (a >> 1) | (u8::from(carry) << 7);
        self.registers.flags.carry = a & 0x01 != 0;
        cycles::RAR
    }

    /// `DAA`: adjusts `A` to two decimal digits after a BCD addition.
    fn daa(&mut self) -> u32 {
        let flags = self.registers.flags;
        (self.registers.a, self.registers.flags) =
            alu::decimal_adjust(self.registers.a, flags.aux_carry, flags.carry);
        cycles::DAA
    }

    /// `CMA`: inverts every bit of `A`. Sets no flags.
    fn cma(&mut self) -> u32 {
        self.registers.a = !self.registers.a;
        cycles::CMA
    }

    /// `STC`: sets `CY` to 1.
    fn stc(&mut self) -> u32 {
        self.registers.flags.carry = true;
        cycles::STC
    }

    /// `CMC`: inverts `CY`.
    fn cmc(&mut self) -> u32 {
        self.registers.flags.carry = !self.registers.flags.carry;
        cycles::CMC
    }

    /// `HLT`: stops the CPU. Later steps execute nothing.
    fn hlt(&mut self) -> u32 {
        self.halted = true;
        cycles::HLT
    }

    /// `MOV dst,src`: copies a byte between registers and `M`.
    fn mov(&mut self, memory: &mut impl MemoryBus, dst: Operand8, src: Operand8) -> u32 {
        let value = self.read_operand(memory, src);
        self.write_operand(memory, dst, value);
        if dst == Operand8::M || src == Operand8::M {
            cycles::MOV_M
        } else {
            cycles::MOV
        }
    }

    /// `ADD src`: adds a register or `M` to `A`.
    fn add(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::add(a, value, false);
        if src == Operand8::M {
            cycles::ADD_M
        } else {
            cycles::ADD
        }
    }

    /// `ADC src`: adds a register or `M` and the carry flag to `A`.
    fn adc(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::add(a, value, self.registers.flags.carry);
        if src == Operand8::M {
            cycles::ADC_M
        } else {
            cycles::ADC
        }
    }

    /// `SUB src`: subtracts a register or `M` from `A`.
    fn sub(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::subtract(a, value, false);
        if src == Operand8::M {
            cycles::SUB_M
        } else {
            cycles::SUB
        }
    }

    /// `SBB src`: subtracts a register or `M` and the carry flag from `A`.
    fn sbb(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) =
            alu::subtract(a, value, self.registers.flags.carry);
        if src == Operand8::M {
            cycles::SBB_M
        } else {
            cycles::SBB
        }
    }

    /// `ANA src`: `A = A AND src`, with a register or `M` as `src`.
    fn ana(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::and(a, value);
        if src == Operand8::M {
            cycles::ANA_M
        } else {
            cycles::ANA
        }
    }

    /// `XRA src`: `A = A XOR src`, with a register or `M` as `src`.
    fn xra(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::xor(a, value);
        if src == Operand8::M {
            cycles::XRA_M
        } else {
            cycles::XRA
        }
    }

    /// `ORA src`: `A = A OR src`, with a register or `M` as `src`.
    fn ora(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (self.registers.a, self.registers.flags) = alu::or(a, value);
        if src == Operand8::M {
            cycles::ORA_M
        } else {
            cycles::ORA
        }
    }

    /// `CMP src`: compares `A` with a register or `M`: sets the flags
    /// from `A - src` and leaves `A` as it was.
    fn cmp(&mut self, memory: &mut impl MemoryBus, src: Operand8) -> u32 {
        let a = self.registers.a;
        let value = self.read_operand(memory, src);
        (_, self.registers.flags) = alu::subtract(a, value, false);
        if src == Operand8::M {
            cycles::CMP_M
        } else {
            cycles::CMP
        }
    }

    /// `XCHG`: swaps `HL` and `DE`.
    fn xchg(&mut self) -> u32 {
        let r = &mut self.registers;
        core::mem::swap(&mut r.h, &mut r.d);
        core::mem::swap(&mut r.l, &mut r.e);
        cycles::XCHG
    }

    /// `ADI value`: adds a constant to `A`.
    fn adi(&mut self, value: u8) -> u32 {
        (self.registers.a, self.registers.flags) = alu::add(self.registers.a, value, false);
        cycles::ADI
    }

    /// `ACI value`: adds a constant and the carry flag to `A`.
    fn aci(&mut self, value: u8) -> u32 {
        let carry = self.registers.flags.carry;
        (self.registers.a, self.registers.flags) = alu::add(self.registers.a, value, carry);
        cycles::ACI
    }

    /// `SUI value`: subtracts a constant from `A`.
    fn sui(&mut self, value: u8) -> u32 {
        (self.registers.a, self.registers.flags) = alu::subtract(self.registers.a, value, false);
        cycles::SUI
    }

    /// `SBI value`: subtracts a constant and the carry flag from `A`.
    fn sbi(&mut self, value: u8) -> u32 {
        let carry = self.registers.flags.carry;
        (self.registers.a, self.registers.flags) = alu::subtract(self.registers.a, value, carry);
        cycles::SBI
    }

    /// `ANI value`: `A = A AND value`.
    fn ani(&mut self, value: u8) -> u32 {
        (self.registers.a, self.registers.flags) = alu::and(self.registers.a, value);
        cycles::ANI
    }

    /// `XRI value`: `A = A XOR value`.
    fn xri(&mut self, value: u8) -> u32 {
        (self.registers.a, self.registers.flags) = alu::xor(self.registers.a, value);
        cycles::XRI
    }

    /// `ORI value`: `A = A OR value`.
    fn ori(&mut self, value: u8) -> u32 {
        (self.registers.a, self.registers.flags) = alu::or(self.registers.a, value);
        cycles::ORI
    }

    /// `CPI value`: compares `A` with a constant: sets the flags from
    /// `A - value` and leaves `A` as it was.
    fn cpi(&mut self, value: u8) -> u32 {
        (_, self.registers.flags) = alu::subtract(self.registers.a, value, false);
        cycles::CPI
    }

    /// Reads an 8-bit operand. `M` reads memory at the address in `HL`.
    fn read_operand(&self, memory: &mut impl MemoryBus, operand: Operand8) -> u8 {
        let r = &self.registers;
        match operand {
            Operand8::B => r.b,
            Operand8::C => r.c,
            Operand8::D => r.d,
            Operand8::E => r.e,
            Operand8::H => r.h,
            Operand8::L => r.l,
            Operand8::M => memory.read(r.hl()),
            Operand8::A => r.a,
        }
    }

    /// Writes an 8-bit operand. `M` writes to memory at the address in `HL`.
    fn write_operand(&mut self, memory: &mut impl MemoryBus, operand: Operand8, value: u8) {
        let r = &mut self.registers;
        match operand {
            Operand8::B => r.b = value,
            Operand8::C => r.c = value,
            Operand8::D => r.d = value,
            Operand8::E => r.e = value,
            Operand8::H => r.h = value,
            Operand8::L => r.l = value,
            Operand8::M => memory.write(r.hl(), value),
            Operand8::A => r.a = value,
        }
    }

    /// Reads a register pair.
    fn read_pair(&self, pair: Operand16) -> u16 {
        let r = &self.registers;
        match pair {
            Operand16::BC => r.bc(),
            Operand16::DE => r.de(),
            Operand16::HL => r.hl(),
            Operand16::SP => r.sp,
        }
    }

    /// Writes a register pair.
    fn write_pair(&mut self, pair: Operand16, value: u16) {
        let r = &mut self.registers;
        match pair {
            Operand16::BC => r.set_bc(value),
            Operand16::DE => r.set_de(value),
            Operand16::HL => r.set_hl(value),
            Operand16::SP => r.sp = value,
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

    /// Every flag at 1, so a test notices when an instruction leaves a flag
    /// set that it should clear.
    const ALL_FLAGS: Flags = Flags {
        sign: true,
        zero: true,
        aux_carry: true,
        parity: true,
        carry: true,
    };

    /// Places `program` at the address in `registers.pc`, gives a new CPU
    /// `registers` and executes one instruction. Returns the CPU and the
    /// cycle count.
    fn run(program: &[u8], registers: Registers) -> (Cpu, u32) {
        let mut memory = [0u8; 0x10000];
        run_with_memory(&mut memory, program, registers)
    }

    /// Same as [`run`], on memory the test fills before the step and checks
    /// after it.
    fn run_with_memory(
        memory: &mut [u8; 0x10000],
        program: &[u8],
        registers: Registers,
    ) -> (Cpu, u32) {
        let start = usize::from(registers.pc);
        memory[start..start + program.len()].copy_from_slice(program);
        let mut cpu = Cpu::new();
        cpu.registers = registers;

        let cycles = cpu.step(memory);

        (cpu, cycles)
    }

    /// Runs one instruction from `program` with `before`. Checks the
    /// registers it leaves behind, flags included, and its cycle count.
    #[track_caller]
    fn assert_registers(program: &[u8], before: Registers, expected: Registers, cycles: u32) {
        let (cpu, actual_cycles) = run(program, before);

        assert_eq!(cpu.registers, expected, "program {program:02X?}");
        assert_eq!(actual_cycles, cycles, "program {program:02X?}");
    }

    /// Same as [`assert_registers`], but ignores the flags. Tests that cover
    /// the flags on their own use it.
    #[track_caller]
    fn assert_registers_except_flags(
        program: &[u8],
        before: Registers,
        expected: Registers,
        cycles: u32,
    ) {
        let (cpu, actual_cycles) = run(program, before);
        let registers = Registers {
            flags: expected.flags,
            ..cpu.registers
        };

        assert_eq!(registers, expected, "program {program:02X?}");
        assert_eq!(actual_cycles, cycles, "program {program:02X?}");
    }

    /// Runs one ALU instruction from `program` on [`distinct_registers`] with
    /// the carry flag at `carry`. Checks that A ends up at `a`, PC moves past
    /// the instruction and the step takes `cycles`.
    #[track_caller]
    fn assert_alu_result(program: &[u8], carry: bool, a: u8, cycles: u32) {
        let registers = Registers {
            flags: Flags {
                carry,
                ..Flags::default()
            },
            ..distinct_registers()
        };

        let next = registers.pc + program.len() as u16;

        let (cpu, actual_cycles) = run(program, registers);

        assert_eq!(cpu.registers.a, a, "program {program:02X?}");
        assert_eq!(cpu.registers.pc, next, "program {program:02X?}");
        assert_eq!(actual_cycles, cycles, "program {program:02X?}");
    }

    /// Runs one ALU instruction from `program` with `registers`. Checks that
    /// A ends up at `a` and the flags at `flags`.
    #[track_caller]
    fn assert_alu_flags(program: &[u8], registers: Registers, a: u8, flags: Flags) {
        let (cpu, _) = run(program, registers);

        assert_eq!(cpu.registers.a, a, "program {program:02X?}, {registers:?}");
        assert_eq!(
            cpu.registers.flags, flags,
            "program {program:02X?}, {registers:?}"
        );
    }

    #[test]
    fn nop_changes_nothing_but_pc() {
        let before = distinct_registers();

        let (cpu, cycles) = run(&[0x00], before); // NOP

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
    fn pc_wraps_from_ffff_to_0000() {
        let registers = Registers {
            pc: 0xFFFF,
            ..distinct_registers()
        };

        let (cpu, _) = run(&[0x00], registers); // NOP

        assert_eq!(cpu.registers.pc, 0x0000);
    }

    #[test]
    fn lxi_loads_constant_into_pair() {
        // Opcodes from the Intel manual, each followed by 1234H low byte
        // first, with the registers they should leave behind.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0103,
            ..before
        };
        let cases = [
            (
                0x01,
                Registers {
                    b: 0x12,
                    c: 0x34,
                    ..after
                },
            ), // LXI B,1234H
            (
                0x11,
                Registers {
                    d: 0x12,
                    e: 0x34,
                    ..after
                },
            ), // LXI D,1234H
            (
                0x21,
                Registers {
                    h: 0x12,
                    l: 0x34,
                    ..after
                },
            ), // LXI H,1234H
            (
                0x31,
                Registers {
                    sp: 0x1234,
                    ..after
                },
            ), // LXI SP,1234H
        ];

        for (opcode, expected) in cases {
            assert_registers(&[opcode, 0x34, 0x12], before, expected, cycles::LXI);
        }
    }

    #[test]
    fn dad_adds_pair_to_hl() {
        // Opcodes from the Intel manual, each with the HL it should leave
        // behind. HL starts at 0102H.
        let cases = [
            (0x09, 0x0C0E), // DAD B: 0102H + 0B0CH
            (0x19, 0x0E10), // DAD D: 0102H + 0D0EH
            (0x29, 0x0204), // DAD H: 0102H + 0102H
            (0x39, 0xF102), // DAD SP: 0102H + F000H
        ];

        for (opcode, hl) in cases {
            let (cpu, cycles) = run(&[opcode], distinct_registers());

            assert_eq!(cpu.registers.hl(), hl, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::DAD, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn dad_sets_only_carry() {
        // Each case: HL, BC and the flags before DAD B, then HL and the
        // flags after. Only CY changes.
        let no_carry = Flags {
            carry: false,
            ..ALL_FLAGS
        };
        let cases = [
            // The DAD example from the Intel manual: A17BH + 339FH = D51AH.
            (0xA17B, 0x339F, ALL_FLAGS, 0xD51A, no_carry),
            // FFFFH + 0001H needs 17 bits, so CY is 1.
            (0xFFFF, 0x0001, no_carry, 0x0000, ALL_FLAGS),
        ];

        for (hl, bc, before, expected_hl, expected_flags) in cases {
            let mut registers = distinct_registers();
            registers.set_hl(hl);
            registers.set_bc(bc);
            registers.flags = before;

            let (cpu, _) = run(&[0x09], registers); // DAD B

            assert_eq!(cpu.registers.hl(), expected_hl, "{hl:#06X} + {bc:#06X}");
            assert_eq!(cpu.registers.flags, expected_flags, "{hl:#06X} + {bc:#06X}");
        }
    }

    #[test]
    fn stax_stores_a_at_pair_address() {
        // Each case: an opcode and the address its pair holds. A holds 0AH.
        let cases = [
            (0x02, 0x0B0C), // STAX B
            (0x12, 0x0D0E), // STAX D
        ];

        for (opcode, address) in cases {
            let mut memory = [0u8; 0x10000];
            let before = distinct_registers();

            let (cpu, cycles) = run_with_memory(&mut memory, &[opcode], before);

            assert_eq!(memory[address], 0x0A, "opcode {opcode:#04X}");
            let after = Registers {
                pc: 0x0101,
                ..before
            };
            assert_eq!(cpu.registers, after, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::STAX, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn ldax_loads_a_from_pair_address() {
        // Each case: an opcode and the byte at the address its pair holds.
        let cases = [
            (0x0A, 0x11), // LDAX B: memory[0B0CH]
            (0x1A, 0x22), // LDAX D: memory[0D0EH]
        ];

        for (opcode, value) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x0B0C] = 0x11;
            memory[0x0D0E] = 0x22;

            let (cpu, cycles) = run_with_memory(&mut memory, &[opcode], distinct_registers());

            assert_eq!(cpu.registers.a, value, "opcode {opcode:#04X}");
            assert_eq!(cpu.registers.pc, 0x0101, "opcode {opcode:#04X}");
            assert_eq!(cycles, cycles::LDAX, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn shld_stores_l_then_h() {
        // The SHLD example from the Intel manual: HL = AE29H, SHLD 010AH.
        let mut memory = [0u8; 0x10000];
        let mut before = distinct_registers();
        before.set_hl(0xAE29);

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x22, 0x0A, 0x01], before);

        assert_eq!(memory[0x010A..0x010C], [0x29, 0xAE]);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0103,
                ..before
            }
        );
        assert_eq!(cycles, cycles::SHLD);
    }

    #[test]
    fn lhld_loads_l_then_h() {
        // The LHLD example from the Intel manual: memory at 025BH holds
        // FFH 03H, so LHLD 025BH loads HL = 03FFH.
        let mut memory = [0u8; 0x10000];
        memory[0x025B..0x025D].copy_from_slice(&[0xFF, 0x03]);
        let before = distinct_registers();

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x2A, 0x5B, 0x02], before);

        assert_eq!(
            cpu.registers,
            Registers {
                h: 0x03,
                l: 0xFF,
                pc: 0x0103,
                ..before
            }
        );
        assert_eq!(cycles, cycles::LHLD);
    }

    #[test]
    fn sta_stores_a_at_address() {
        let mut memory = [0u8; 0x10000];
        let before = distinct_registers();

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x32, 0x00, 0x20], before); // STA 2000H

        assert_eq!(memory[0x2000], 0x0A);
        assert_eq!(
            cpu.registers,
            Registers {
                pc: 0x0103,
                ..before
            }
        );
        assert_eq!(cycles, cycles::STA);
    }

    #[test]
    fn lda_loads_a_from_address() {
        let mut memory = [0u8; 0x10000];
        memory[0x2000] = 0x99;
        let before = distinct_registers();

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x3A, 0x00, 0x20], before); // LDA 2000H

        assert_eq!(
            cpu.registers,
            Registers {
                a: 0x99,
                pc: 0x0103,
                ..before
            }
        );
        assert_eq!(cycles, cycles::LDA);
    }

    #[test]
    fn inx_increments_pair() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the pair grew by 1 and PC moved 1 byte.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0101,
            ..before
        };
        let cases = [
            (0x03, Registers { c: 0x0D, ..after }), // INX B: 0B0CH + 1
            (0x13, Registers { e: 0x0F, ..after }), // INX D: 0D0EH + 1
            (0x23, Registers { l: 0x03, ..after }), // INX H: 0102H + 1
            (
                0x33,
                Registers {
                    sp: 0xF001,
                    ..after
                },
            ), // INX SP: F000H + 1
        ];

        for (opcode, expected) in cases {
            assert_registers(&[opcode], before, expected, cycles::INX);
        }
    }

    #[test]
    fn dcx_decrements_pair() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the pair shrank by 1 and PC moved 1 byte.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0101,
            ..before
        };
        let cases = [
            (0x0B, Registers { c: 0x0B, ..after }), // DCX B: 0B0CH - 1
            (0x1B, Registers { e: 0x0D, ..after }), // DCX D: 0D0EH - 1
            (0x2B, Registers { l: 0x01, ..after }), // DCX H: 0102H - 1
            (
                0x3B,
                Registers {
                    sp: 0xEFFF,
                    ..after
                },
            ), // DCX SP: F000H - 1
        ];

        for (opcode, expected) in cases {
            assert_registers(&[opcode], before, expected, cycles::DCX);
        }
    }

    #[test]
    fn inx_and_dcx_wrap_and_set_no_flags() {
        // Each case: an opcode, HL before and after. HL wraps around, and
        // the flags stay at 0 even when HL reaches 0000H.
        let cases = [
            (0x23, 0xFFFF, 0x0000), // INX H
            (0x2B, 0x0000, 0xFFFF), // DCX H
        ];

        for (opcode, hl, expected) in cases {
            let mut registers = distinct_registers();
            registers.set_hl(hl);

            let (cpu, _) = run(&[opcode], registers);

            assert_eq!(cpu.registers.hl(), expected, "opcode {opcode:#04X}");
            assert_eq!(
                cpu.registers.flags,
                Flags::default(),
                "opcode {opcode:#04X}"
            );
        }
    }

    #[test]
    fn inr_increments_register() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the register grew by 1 and PC moved 1 byte.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0101,
            ..before
        };
        let cases = [
            (0x04, Registers { b: 0x0C, ..after }), // INR B
            (0x0C, Registers { c: 0x0D, ..after }), // INR C
            (0x14, Registers { d: 0x0E, ..after }), // INR D
            (0x1C, Registers { e: 0x0F, ..after }), // INR E
            (0x24, Registers { h: 0x02, ..after }), // INR H
            (0x2C, Registers { l: 0x03, ..after }), // INR L
            (0x3C, Registers { a: 0x0B, ..after }), // INR A
        ];

        // The flags have their own test, so this one ignores them.
        for (opcode, expected) in cases {
            assert_registers_except_flags(&[opcode], before, expected, cycles::INR);
        }
    }

    #[test]
    fn inr_sets_flags_and_keeps_carry() {
        // Each case: the flags before the step, A, the result and the flags
        // INR A should leave behind. CY keeps its value in every case.
        let none = Flags::default();
        let cases = [
            (
                ALL_FLAGS,
                0x01,
                0x02,
                Flags {
                    carry: true,
                    ..none
                },
            ),
            (
                ALL_FLAGS,
                0x0F,
                0x10,
                Flags {
                    aux_carry: true,
                    carry: true,
                    ..none
                },
            ),
            (
                ALL_FLAGS,
                0x7F,
                0x80,
                Flags {
                    sign: true,
                    aux_carry: true,
                    carry: true,
                    ..none
                },
            ),
            // FFH wraps to 00H, and CY still stays 0.
            (
                none,
                0xFF,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
        ];

        for (before, a, result, flags) in cases {
            let registers = Registers {
                a,
                flags: before,
                ..distinct_registers()
            };

            assert_alu_flags(&[0x3C], registers, result, flags); // INR A
        }
    }

    #[test]
    fn dcr_decrements_register() {
        // Opcodes from the Intel manual, each with the registers it should
        // leave behind: the register shrank by 1 and PC moved 1 byte.
        let before = distinct_registers();
        let after = Registers {
            pc: 0x0101,
            ..before
        };
        let cases = [
            (0x05, Registers { b: 0x0A, ..after }), // DCR B
            (0x0D, Registers { c: 0x0B, ..after }), // DCR C
            (0x15, Registers { d: 0x0C, ..after }), // DCR D
            (0x1D, Registers { e: 0x0D, ..after }), // DCR E
            (0x25, Registers { h: 0x00, ..after }), // DCR H
            (0x2D, Registers { l: 0x01, ..after }), // DCR L
            (0x3D, Registers { a: 0x09, ..after }), // DCR A
        ];

        // The flags have their own test, so this one ignores them.
        for (opcode, expected) in cases {
            assert_registers_except_flags(&[opcode], before, expected, cycles::DCR);
        }
    }

    #[test]
    fn dcr_sets_flags_and_keeps_carry() {
        // Each case: the flags before the step, A, the result and the flags
        // DCR A should leave behind. CY keeps its value in every case.
        let none = Flags::default();
        let cases = [
            // 10H - 1 borrows from the high 4 bits, so AC is 0.
            (
                ALL_FLAGS,
                0x10,
                0x0F,
                Flags {
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            (
                ALL_FLAGS,
                0x01,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            (
                ALL_FLAGS,
                0x80,
                0x7F,
                Flags {
                    carry: true,
                    ..none
                },
            ),
            // 00H wraps to FFH, and CY still stays 0.
            (
                none,
                0x00,
                0xFF,
                Flags {
                    sign: true,
                    parity: true,
                    ..none
                },
            ),
        ];

        for (before, a, result, flags) in cases {
            let registers = Registers {
                a,
                flags: before,
                ..distinct_registers()
            };

            assert_alu_flags(&[0x3D], registers, result, flags); // DCR A
        }
    }

    #[test]
    fn inr_and_dcr_m_change_memory_at_hl() {
        // Each case: an opcode with M as the operand, the byte it should
        // leave at HL and its cycle count. Memory at HL holds 40H, the
        // example from the Intel manual's DCR M.
        let cases = [
            (0x34, 0x41, cycles::INR_M), // INR M
            (0x35, 0x3F, cycles::DCR_M), // DCR M
        ];

        for (opcode, result, expected_cycles) in cases {
            let mut memory = [0u8; 0x10000];
            memory[0x3A7C] = 0x40;
            let registers = Registers {
                h: 0x3A,
                l: 0x7C,
                ..distinct_registers()
            };

            let (_, cycles) = run_with_memory(&mut memory, &[opcode], registers);

            assert_eq!(memory[0x3A7C], result, "opcode {opcode:#04X}");
            assert_eq!(cycles, expected_cycles, "opcode {opcode:#04X}");
        }
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
            assert_registers(&[opcode, 0x42], before, expected, cycles::MVI);
        }
    }

    #[test]
    fn mvi_m_stores_next_byte_at_hl() {
        let mut memory = [0u8; 0x10000];
        let before = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x36, 0x99], before); // MVI M,99H

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
    fn accumulator_ops_change_a() {
        // Each case: an opcode, the carry flag before the step, the value it
        // should leave in A and its cycle count. A starts at 0AH, 0000 1010.
        let cases = [
            (0x07, false, 0x14, cycles::RLC), // RLC: 0001 0100
            (0x0F, false, 0x05, cycles::RRC), // RRC: 0000 0101
            (0x17, true, 0x15, cycles::RAL),  // RAL: 0001 0101, old CY in bit 0
            (0x1F, true, 0x85, cycles::RAR),  // RAR: 1000 0101, old CY in bit 7
            (0x27, false, 0x10, cycles::DAA), // DAA: 0AH is not BCD, 0AH + 06H
            (0x2F, false, 0xF5, cycles::CMA), // CMA: 1111 0101
            (0x37, false, 0x0A, cycles::STC), // STC: A stays
            (0x3F, false, 0x0A, cycles::CMC), // CMC: A stays
        ];

        for (opcode, carry, result, cycles) in cases {
            assert_alu_result(&[opcode], carry, result, cycles);
        }
    }

    #[test]
    fn rotates_and_carry_ops_set_only_carry() {
        // Each case: an opcode, A and CY before the step, then A and CY
        // after it. The other flags start at 1 and stay there. The first
        // case for each rotate repeats the example from the Intel manual.
        let cases = [
            (0x07, 0xF2, false, 0xE5, true),  // RLC
            (0x07, 0x01, true, 0x02, false),  // RLC ignores the old CY
            (0x0F, 0xF2, false, 0x79, false), // RRC
            (0x0F, 0x01, false, 0x80, true),  // RRC
            (0x17, 0xB5, false, 0x6A, true),  // RAL
            (0x17, 0x00, true, 0x01, false),  // RAL moves the old CY into bit 0
            (0x1F, 0x6A, true, 0xB5, false),  // RAR
            (0x1F, 0x01, false, 0x00, true),  // RAR
            (0x2F, 0x51, false, 0xAE, false), // CMA leaves CY alone
            (0x37, 0x0A, false, 0x0A, true),  // STC
            (0x37, 0x0A, true, 0x0A, true),   // STC
            (0x3F, 0x0A, false, 0x0A, true),  // CMC
            (0x3F, 0x0A, true, 0x0A, false),  // CMC
        ];

        for (opcode, a, carry, result, carry_out) in cases {
            let registers = Registers {
                a,
                flags: Flags { carry, ..ALL_FLAGS },
                ..distinct_registers()
            };
            let flags = Flags {
                carry: carry_out,
                ..ALL_FLAGS
            };

            assert_alu_flags(&[opcode], registers, result, flags);
        }
    }

    #[test]
    fn daa_adjusts_a_to_two_decimal_digits() {
        // Each case: A, AC and CY that an addition left, then A and the
        // flags DAA should leave behind.
        let none = Flags::default();
        let cases = [
            // The DAA example from the Intel manual: 9BH turns into 01H
            // with CY = 1.
            (
                0x9B,
                none,
                0x01,
                Flags {
                    aux_carry: true,
                    carry: true,
                    ..none
                },
            ),
            // 15H + 27H = 3CH: the low digit is above 9.
            (
                0x3C,
                none,
                0x42,
                Flags {
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            // 19H + 28H = 41H with AC = 1: the low digit overflowed.
            (
                0x41,
                Flags {
                    aux_carry: true,
                    ..none
                },
                0x47,
                Flags {
                    parity: true,
                    ..none
                },
            ),
            // 90H + 90H = 20H with CY = 1: the high digit overflowed.
            (
                0x20,
                Flags {
                    carry: true,
                    ..none
                },
                0x80,
                Flags {
                    sign: true,
                    carry: true,
                    ..none
                },
            ),
            // 99H + 01H = 9AH: both digits roll over, 99 + 1 = 100.
            (
                0x9A,
                none,
                0x00,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
            // 42H is already BCD, so DAA adds nothing.
            (
                0x42,
                none,
                0x42,
                Flags {
                    parity: true,
                    ..none
                },
            ),
        ];

        for (a, before, result, flags) in cases {
            let registers = Registers {
                a,
                flags: before,
                ..distinct_registers()
            };

            assert_alu_flags(&[0x27], registers, result, flags); // DAA
        }
    }

    #[test]
    fn hlt_halts_cpu() {
        let before = distinct_registers();

        let (cpu, cycles) = run(&[0x76], before); // HLT

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
            assert_registers(&[opcode], before, expected, cycles::MOV);
        }
    }

    #[test]
    fn mov_from_m_reads_memory_at_hl() {
        let mut memory = [0u8; 0x10000];
        memory[0x2000] = 0x99;
        let before = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x7E], before); // MOV A,M

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
        let before = Registers {
            h: 0x20,
            l: 0x00,
            ..distinct_registers()
        };

        let (cpu, cycles) = run_with_memory(&mut memory, &[0x77], before); // MOV M,A

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
            assert_alu_result(&[opcode], false, sum, cycles::ADD);
        }
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
            let registers = Registers {
                a,
                b,
                flags: ALL_FLAGS,
                ..distinct_registers()
            };

            assert_alu_flags(&[0x80], registers, sum, flags); // ADD B
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
            assert_alu_result(&[opcode], carry, sum, cycles::ADC);
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
            let registers = Registers {
                a,
                b,
                flags: Flags { carry, ..none },
                ..distinct_registers()
            };

            assert_alu_flags(&[0x88], registers, sum, flags); // ADC B
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
            assert_alu_result(&[opcode], false, difference, cycles::SUB);
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
            let registers = Registers {
                a,
                b,
                flags: ALL_FLAGS,
                ..distinct_registers()
            };

            assert_alu_flags(&[0x90], registers, difference, flags); // SUB B
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
            assert_alu_result(&[opcode], carry, difference, cycles::SBB);
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
            let registers = Registers {
                a,
                l,
                flags: Flags { carry, ..none },
                ..distinct_registers()
            };

            assert_alu_flags(&[0x9D], registers, difference, flags); // SBB L
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
            assert_alu_result(&[opcode], false, result, cycles::ANA);
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
            let registers = Registers {
                a,
                b,
                flags: ALL_FLAGS,
                ..distinct_registers()
            };

            assert_alu_flags(&[0xA0], registers, result, flags); // ANA B
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
            assert_alu_result(&[opcode], false, result, cycles::XRA);
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
            let registers = Registers {
                a,
                b,
                flags: ALL_FLAGS,
                ..distinct_registers()
            };

            assert_alu_flags(&[0xA8], registers, result, flags); // XRA B
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
            assert_alu_result(&[opcode], false, result, cycles::ORA);
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
            let registers = Registers {
                a,
                b,
                flags: ALL_FLAGS,
                ..distinct_registers()
            };

            assert_alu_flags(&[0xB0], registers, result, flags); // ORA B
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
            let (cpu, cycles) = run(&[opcode], distinct_registers());

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
            let registers = Registers {
                a,
                e,
                ..distinct_registers()
            };

            assert_alu_flags(&[0xBB], registers, a, flags); // CMP E
        }
    }

    #[test]
    fn alu_with_m_reads_memory_at_hl() {
        // Each case: an opcode with M as the source, the value it should
        // leave in A and its cycle count. A starts at 10H, memory at HL
        // holds 05H and the carry flag is 1.
        let cases = [
            (0x86, 0x15, cycles::ADD_M), // ADD M: 10H + 05H
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
            memory[0x2000] = 0x05;
            let mut registers = Registers {
                a: 0x10,
                h: 0x20,
                l: 0x00,
                ..distinct_registers()
            };
            registers.flags.carry = true;

            let (cpu, cycles) = run_with_memory(&mut memory, &[opcode], registers);

            assert_eq!(cpu.registers.a, result, "opcode {opcode:#04X}");
            assert_eq!(cycles, expected_cycles, "opcode {opcode:#04X}");
        }
    }

    #[test]
    fn xchg_swaps_hl_and_de() {
        let before = distinct_registers();

        let (cpu, cycles) = run(&[0xEB], before); // XCHG

        assert_eq!(
            cpu.registers,
            Registers {
                d: before.h,
                e: before.l,
                h: before.d,
                l: before.e,
                pc: 0x0101,
                ..before
            }
        );
        assert_eq!(cycles, cycles::XCHG);
    }

    #[test]
    fn immediate_alu_uses_next_byte() {
        // Each case: an opcode with the constant 03H after it, the value it
        // should leave in A and its cycle count. A starts at 0AH and the
        // carry flag is 1.
        let cases = [
            (0xC6, 0x0D, cycles::ADI), // ADI 03H: 0AH + 03H
            (0xCE, 0x0E, cycles::ACI), // ACI 03H: 0AH + 03H + 1
            (0xD6, 0x07, cycles::SUI), // SUI 03H: 0AH - 03H
            (0xDE, 0x06, cycles::SBI), // SBI 03H: 0AH - 03H - 1
            (0xE6, 0x02, cycles::ANI), // ANI 03H: 0AH AND 03H
            (0xEE, 0x09, cycles::XRI), // XRI 03H: 0AH XOR 03H
            (0xF6, 0x0B, cycles::ORI), // ORI 03H: 0AH OR 03H
            (0xFE, 0x0A, cycles::CPI), // CPI 03H: A stays 0AH
        ];

        for (opcode, result, cycles) in cases {
            assert_alu_result(&[opcode, 0x03], true, result, cycles);
        }
    }

    #[test]
    fn cpi_sets_flags_like_cmp() {
        // Each case: the constant and the flags CPI should leave behind. A
        // holds 4AH. The first case repeats the example from the Intel
        // manual.
        let none = Flags::default();
        let cases = [
            (
                0x40,
                Flags {
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            (
                0x4A,
                Flags {
                    zero: true,
                    aux_carry: true,
                    parity: true,
                    ..none
                },
            ),
            (
                0x50,
                Flags {
                    sign: true,
                    aux_carry: true,
                    parity: true,
                    carry: true,
                    ..none
                },
            ),
        ];

        for (value, flags) in cases {
            let registers = Registers {
                a: 0x4A,
                ..distinct_registers()
            };

            assert_alu_flags(&[0xFE, value], registers, 0x4A, flags); // CPI value
        }
    }

    #[test]
    #[should_panic(expected = "opcode 0xC3 is not supported yet")]
    fn unsupported_opcode_panics() {
        run(&[0xC3, 0x00, 0x00], distinct_registers()); // JMP 0000H, not implemented yet
    }
}
