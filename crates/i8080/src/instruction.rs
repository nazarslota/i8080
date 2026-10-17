use crate::condition::Condition;

/// One decoded 8080 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    /// `NOP`: does nothing.
    Nop,
    /// `LXI dst,value`: loads the 16-bit constant that follows the opcode
    /// into `dst`.
    Lxi { dst: Operand16, value: u16 },
    /// `STAX dst`: stores `A` at the address in `dst`, `BC` or `DE`.
    Stax { dst: Operand16 },
    /// `INX pair`: adds 1 to `pair`.
    Inx { pair: Operand16 },
    /// `INR dst`: adds 1 to `dst` and leaves the carry flag alone.
    Inr { dst: Operand8 },
    /// `DCR dst`: subtracts 1 from `dst` and leaves the carry flag alone.
    Dcr { dst: Operand8 },
    /// `MVI dst,value`: loads the byte that follows the opcode into `dst`.
    Mvi { dst: Operand8, value: u8 },
    /// `RLC`: rotates `A` left. Bit 7 goes to bit 0 and to `CY`.
    Rlc,
    /// `DAD src`: adds `src` to `HL`.
    Dad { src: Operand16 },
    /// `LDAX src`: loads `A` from the address in `src`, `BC` or `DE`.
    Ldax { src: Operand16 },
    /// `DCX pair`: subtracts 1 from `pair`.
    Dcx { pair: Operand16 },
    /// `RRC`: rotates `A` right. Bit 0 goes to bit 7 and to `CY`.
    Rrc,
    /// `RAL`: rotates `A` left through `CY`.
    Ral,
    /// `RAR`: rotates `A` right through `CY`.
    Rar,
    /// `SHLD address`: stores `L` at `address` and `H` at `address + 1`.
    Shld { address: u16 },
    /// `DAA`: adjusts `A` to two decimal digits after a BCD addition.
    Daa,
    /// `LHLD address`: loads `L` from `address` and `H` from `address + 1`.
    Lhld { address: u16 },
    /// `CMA`: inverts every bit of `A`.
    Cma,
    /// `STA address`: stores `A` at `address`.
    Sta { address: u16 },
    /// `STC`: sets `CY` to 1.
    Stc,
    /// `LDA address`: loads `A` from `address`.
    Lda { address: u16 },
    /// `CMC`: inverts `CY`.
    Cmc,
    /// `HLT`: stops the CPU until an interrupt or a reset.
    Hlt,
    /// `MOV dst,src`: copies `src` into `dst`.
    Mov { dst: Operand8, src: Operand8 },
    /// `ADD src`: adds `src` to `A`.
    Add { src: Operand8 },
    /// `ADC src`: adds `src` and the carry flag to `A`.
    Adc { src: Operand8 },
    /// `SUB src`: subtracts `src` from `A`.
    Sub { src: Operand8 },
    /// `SBB src`: subtracts `src` and the carry flag from `A`.
    Sbb { src: Operand8 },
    /// `ANA src`: `A = A AND src`.
    Ana { src: Operand8 },
    /// `XRA src`: `A = A XOR src`.
    Xra { src: Operand8 },
    /// `ORA src`: `A = A OR src`.
    Ora { src: Operand8 },
    /// `CMP src`: sets the flags from `A - src` and leaves `A` as it was.
    Cmp { src: Operand8 },
    /// `Jcc address`: jumps to `address` if `condition` holds.
    Jcc { condition: Condition, address: u16 },
    /// `JMP address`: jumps to `address`.
    Jmp { address: u16 },
    /// `ADI value`: adds a constant to `A`.
    Adi { value: u8 },
    /// `ACI value`: adds a constant and the carry flag to `A`.
    Aci { value: u8 },
    /// `SUI value`: subtracts a constant from `A`.
    Sui { value: u8 },
    /// `SBI value`: subtracts a constant and the carry flag from `A`.
    Sbi { value: u8 },
    /// `ANI value`: `A = A AND value`.
    Ani { value: u8 },
    /// `PCHL`: jumps to the address in `HL`.
    Pchl,
    /// `XCHG`: swaps `HL` and `DE`.
    Xchg,
    /// `XRI value`: `A = A XOR value`.
    Xri { value: u8 },
    /// `ORI value`: `A = A OR value`.
    Ori { value: u8 },
    /// `CPI value`: sets the flags from `A - value` and leaves `A` as it
    /// was.
    Cpi { value: u8 },
}

impl Instruction {
    /// Decodes one instruction.
    ///
    /// `next_byte` returns the instruction's bytes in order, opcode first.
    /// Decoding calls it once for each byte the instruction has, the way the
    /// 8080 reads an instruction one byte at a time.
    ///
    /// # Panics
    ///
    /// Panics on opcodes the emulator doesn't support yet. Once all 256
    /// opcodes are in, decoding can't fail.
    pub(crate) fn decode(mut next_byte: impl FnMut() -> u8) -> Self {
        let opcode = next_byte();

        // Many opcodes carry their operand in a bit field. The Intel manual
        // names the fields `ddd` (destination, bits 5-3), `sss` (source,
        // bits 2-0), `rp` (register pair, bits 5-4) and `ccc` (condition,
        // bits 5-3). Each `from_code` keeps the low bits after the shift.
        //
        //   MVI A,n  = 3EH = 00 111 110   ddd = 111, A
        //   LXI H,nn = 21H = 00 10 0001   rp  = 10, HL
        let ddd = Operand8::from_code(opcode >> 3);
        let sss = Operand8::from_code(opcode);
        let rp = Operand16::from_code(opcode >> 4);
        let ccc = Condition::from_code(opcode >> 3);

        match opcode {
            // Intel documents only 00H. The 8080 runs the other seven as
            // NOP too.
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => Self::Nop,
            0x01 | 0x11 | 0x21 | 0x31 => Self::Lxi {
                dst: rp,
                value: next_word(&mut next_byte),
            },
            0x02 | 0x12 => Self::Stax { dst: rp },
            0x03 | 0x13 | 0x23 | 0x33 => Self::Inx { pair: rp },
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => Self::Inr { dst: ddd },
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => Self::Dcr { dst: ddd },
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => Self::Mvi {
                dst: ddd,
                value: next_byte(),
            },
            0x07 => Self::Rlc,
            0x09 | 0x19 | 0x29 | 0x39 => Self::Dad { src: rp },
            0x0A | 0x1A => Self::Ldax { src: rp },
            0x0B | 0x1B | 0x2B | 0x3B => Self::Dcx { pair: rp },
            0x0F => Self::Rrc,
            0x17 => Self::Ral,
            0x1F => Self::Rar,
            0x22 => Self::Shld {
                address: next_word(&mut next_byte),
            },
            0x27 => Self::Daa,
            0x2A => Self::Lhld {
                address: next_word(&mut next_byte),
            },
            0x2F => Self::Cma,
            0x32 => Self::Sta {
                address: next_word(&mut next_byte),
            },
            0x37 => Self::Stc,
            0x3A => Self::Lda {
                address: next_word(&mut next_byte),
            },
            0x3F => Self::Cmc,
            // HLT takes the slot where MOV M,M would be, so it comes before
            // the MOV range.
            0x76 => Self::Hlt,
            0x40..=0x7F => Self::Mov { dst: ddd, src: sss },
            0x80..=0x87 => Self::Add { src: sss },
            0x88..=0x8F => Self::Adc { src: sss },
            0x90..=0x97 => Self::Sub { src: sss },
            0x98..=0x9F => Self::Sbb { src: sss },
            0xA0..=0xA7 => Self::Ana { src: sss },
            0xA8..=0xAF => Self::Xra { src: sss },
            0xB0..=0xB7 => Self::Ora { src: sss },
            0xB8..=0xBF => Self::Cmp { src: sss },
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => Self::Jcc {
                condition: ccc,
                address: next_word(&mut next_byte),
            },
            0xC3 => Self::Jmp {
                address: next_word(&mut next_byte),
            },
            0xC6 => Self::Adi { value: next_byte() },
            0xCE => Self::Aci { value: next_byte() },
            0xD6 => Self::Sui { value: next_byte() },
            0xDE => Self::Sbi { value: next_byte() },
            0xE6 => Self::Ani { value: next_byte() },
            0xE9 => Self::Pchl,
            0xEB => Self::Xchg,
            0xEE => Self::Xri { value: next_byte() },
            0xF6 => Self::Ori { value: next_byte() },
            0xFE => Self::Cpi { value: next_byte() },
            _ => unimplemented!("opcode {opcode:#04X} is not supported yet"),
        }
    }
}

/// Reads a 16-bit constant. The 8080 stores it after the opcode low byte
/// first: `LXI H,2000H` assembles to `21 00 20`.
fn next_word(next_byte: &mut impl FnMut() -> u8) -> u16 {
    let low = next_byte();
    let high = next_byte();
    u16::from_le_bytes([low, high])
}

/// An 8-bit operand, as opcodes encode it in a 3-bit field.
///
/// The discriminants follow the Intel manual: `B` is 000, `C` is 001 and so
/// on up to `A` at 111. `M` stands for the memory byte at the address in
/// `HL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operand8 {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    M = 6,
    A = 7,
}

impl Operand8 {
    /// Decodes a 3-bit operand field. It ignores bits above the lowest
    /// three.
    pub(crate) const fn from_code(code: u8) -> Self {
        match code & 0b111 {
            0 => Self::B,
            1 => Self::C,
            2 => Self::D,
            3 => Self::E,
            4 => Self::H,
            5 => Self::L,
            6 => Self::M,
            _ => Self::A,
        }
    }
}

/// A 16-bit operand: a register pair, as opcodes encode it in a 2-bit
/// field.
///
/// The discriminants follow the Intel manual: `BC` is 00, `DE` 01, `HL` 10
/// and `SP` 11. Intel assembler names a pair by its first register:
/// `LXI B` loads `BC`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operand16 {
    BC = 0,
    DE = 1,
    HL = 2,
    SP = 3,
}

impl Operand16 {
    /// Decodes a 2-bit register pair field. It ignores bits above the
    /// lowest two.
    pub(crate) const fn from_code(code: u8) -> Self {
        match code & 0b11 {
            0 => Self::BC,
            1 => Self::DE,
            2 => Self::HL,
            _ => Self::SP,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operand8_codes_match_intel_manual() {
        let manual_order = [
            Operand8::B,
            Operand8::C,
            Operand8::D,
            Operand8::E,
            Operand8::H,
            Operand8::L,
            Operand8::M,
            Operand8::A,
        ];

        for (code, operand) in (0..).zip(manual_order) {
            assert_eq!(Operand8::from_code(code), operand, "code {code:03b}");
        }
    }

    #[test]
    fn operand16_codes_match_intel_manual() {
        let manual_order = [Operand16::BC, Operand16::DE, Operand16::HL, Operand16::SP];

        for (code, pair) in (0..).zip(manual_order) {
            assert_eq!(Operand16::from_code(code), pair, "code {code:02b}");
        }
    }

    #[test]
    fn from_code_ignores_bits_above_the_field() {
        assert_eq!(Operand8::from_code(0b1111_1000), Operand8::B);
        assert_eq!(Operand8::from_code(0b0011_1111), Operand8::A);
    }
}
