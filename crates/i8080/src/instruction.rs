/// One decoded 8080 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    /// `NOP`: does nothing.
    Nop,
    /// `LXI dst,value`: loads the 16-bit constant that follows the opcode
    /// into `dst`.
    Lxi { dst: Operand16, value: u16 },
    /// `DAD src`: adds `src` to `HL`.
    Dad { src: Operand16 },
    /// `STAX dst`: stores `A` at the address in `dst`, `BC` or `DE`.
    Stax { dst: Operand16 },
    /// `LDAX src`: loads `A` from the address in `src`, `BC` or `DE`.
    Ldax { src: Operand16 },
    /// `SHLD address`: stores `L` at `address` and `H` at `address + 1`.
    Shld { address: u16 },
    /// `LHLD address`: loads `L` from `address` and `H` from `address + 1`.
    Lhld { address: u16 },
    /// `STA address`: stores `A` at `address`.
    Sta { address: u16 },
    /// `LDA address`: loads `A` from `address`.
    Lda { address: u16 },
    /// `INX pair`: adds 1 to `pair`.
    Inx { pair: Operand16 },
    /// `DCX pair`: subtracts 1 from `pair`.
    Dcx { pair: Operand16 },
    /// `INR dst`: adds 1 to `dst` and leaves the carry flag alone.
    Inr { dst: Operand8 },
    /// `DCR dst`: subtracts 1 from `dst` and leaves the carry flag alone.
    Dcr { dst: Operand8 },
    /// `MVI dst,value`: loads the byte that follows the opcode into `dst`.
    Mvi { dst: Operand8, value: u8 },
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
    /// `XCHG`: swaps `HL` and `DE`.
    Xchg,
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

        // Split the opcode into three bit fields, xx yyy zzz. The Intel
        // manual calls yyy `ddd` when it names a destination operand and
        // zzz `sss` when it names a source operand.
        //
        //   MVI A,n = 3EH = 00 111 110
        //                   xx yyy zzz   x = 00, y = 111 (A), z = 110
        //
        // Shifting right drops the bits below a field, and `& 0b111` keeps
        // the three lowest bits that remain.
        let x = opcode >> 6;
        let y = (opcode >> 3) & 0b111;
        let z = opcode & 0b111;

        match (x, y, z) {
            // NOP: 00 000 000
            (0b00, 0b000, 0b000) => Self::Nop,
            // LXI rp,nn: 00 rp0 001. The two bits rp sit above bit 3, so
            // `y >> 1` picks them and bit 3 tells LXI from DAD.
            (0b00, y, 0b001) if y & 1 == 0 => Self::Lxi {
                dst: Operand16::from_code(y >> 1),
                value: next_word(&mut next_byte),
            },
            // DAD rp: 00 rp1 001
            (0b00, y, 0b001) => Self::Dad {
                src: Operand16::from_code(y >> 1),
            },
            // STAX rp: 00 rp0 010, for BC and DE only
            (0b00, y @ (0b000 | 0b010), 0b010) => Self::Stax {
                dst: Operand16::from_code(y >> 1),
            },
            // LDAX rp: 00 rp1 010, for BC and DE only
            (0b00, y @ (0b001 | 0b011), 0b010) => Self::Ldax {
                src: Operand16::from_code(y >> 1),
            },
            // The slots STAX H, LDAX H, STAX SP and LDAX SP would take
            // hold the instructions with a 16-bit address instead.
            // SHLD nn: 00 100 010
            (0b00, 0b100, 0b010) => Self::Shld {
                address: next_word(&mut next_byte),
            },
            // LHLD nn: 00 101 010
            (0b00, 0b101, 0b010) => Self::Lhld {
                address: next_word(&mut next_byte),
            },
            // STA nn: 00 110 010
            (0b00, 0b110, 0b010) => Self::Sta {
                address: next_word(&mut next_byte),
            },
            // LDA nn: 00 111 010
            (0b00, 0b111, 0b010) => Self::Lda {
                address: next_word(&mut next_byte),
            },
            // INX rp: 00 rp0 011
            (0b00, y, 0b011) if y & 1 == 0 => Self::Inx {
                pair: Operand16::from_code(y >> 1),
            },
            // DCX rp: 00 rp1 011
            (0b00, y, 0b011) => Self::Dcx {
                pair: Operand16::from_code(y >> 1),
            },
            // INR ddd: 00 ddd 100
            (0b00, ddd, 0b100) => Self::Inr {
                dst: Operand8::from_code(ddd),
            },
            // DCR ddd: 00 ddd 101
            (0b00, ddd, 0b101) => Self::Dcr {
                dst: Operand8::from_code(ddd),
            },
            // MVI ddd,n: 00 ddd 110
            (0b00, ddd, 0b110) => Self::Mvi {
                dst: Operand8::from_code(ddd),
                value: next_byte(),
            },
            // HLT: 01 110 110, the slot where MOV M,M would be. It comes
            // before the MOV pattern, so `match` picks it first.
            (0b01, 0b110, 0b110) => Self::Hlt,
            // MOV ddd,sss: 01 ddd sss
            (0b01, ddd, sss) => Self::Mov {
                dst: Operand8::from_code(ddd),
                src: Operand8::from_code(sss),
            },
            // ADD sss: 10 000 sss
            (0b10, 0b000, sss) => Self::Add {
                src: Operand8::from_code(sss),
            },
            // ADC sss: 10 001 sss
            (0b10, 0b001, sss) => Self::Adc {
                src: Operand8::from_code(sss),
            },
            // SUB sss: 10 010 sss
            (0b10, 0b010, sss) => Self::Sub {
                src: Operand8::from_code(sss),
            },
            // SBB sss: 10 011 sss
            (0b10, 0b011, sss) => Self::Sbb {
                src: Operand8::from_code(sss),
            },
            // ANA sss: 10 100 sss
            (0b10, 0b100, sss) => Self::Ana {
                src: Operand8::from_code(sss),
            },
            // XRA sss: 10 101 sss
            (0b10, 0b101, sss) => Self::Xra {
                src: Operand8::from_code(sss),
            },
            // ORA sss: 10 110 sss
            (0b10, 0b110, sss) => Self::Ora {
                src: Operand8::from_code(sss),
            },
            // CMP sss: 10 111 sss
            (0b10, 0b111, sss) => Self::Cmp {
                src: Operand8::from_code(sss),
            },
            // XCHG: 11 101 011
            (0b11, 0b101, 0b011) => Self::Xchg,
            // The same eight operations with a constant byte after the
            // opcode: 11 yyy 110.
            (0b11, 0b000, 0b110) => Self::Adi { value: next_byte() },
            (0b11, 0b001, 0b110) => Self::Aci { value: next_byte() },
            (0b11, 0b010, 0b110) => Self::Sui { value: next_byte() },
            (0b11, 0b011, 0b110) => Self::Sbi { value: next_byte() },
            (0b11, 0b100, 0b110) => Self::Ani { value: next_byte() },
            (0b11, 0b101, 0b110) => Self::Xri { value: next_byte() },
            (0b11, 0b110, 0b110) => Self::Ori { value: next_byte() },
            (0b11, 0b111, 0b110) => Self::Cpi { value: next_byte() },
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
