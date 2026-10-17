/// One decoded 8080 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    /// `NOP`: does nothing.
    Nop,
    /// `MVI dst,value`: loads the byte that follows the opcode into `dst`.
    Mvi { dst: Operand, value: u8 },
    /// `MOV dst,src`: copies `src` into `dst`.
    Mov { dst: Operand, src: Operand },
    /// `HLT`: stops the CPU until an interrupt or a reset.
    Hlt,
    /// `ADD src`: adds `src` to `A`.
    Add { src: Operand },
    /// `ADC src`: adds `src` and the carry flag to `A`.
    Adc { src: Operand },
    /// `SUB src`: subtracts `src` from `A`.
    Sub { src: Operand },
    /// `SBB src`: subtracts `src` and the carry flag from `A`.
    Sbb { src: Operand },
    /// `ANA src`: `A = A AND src`.
    Ana { src: Operand },
    /// `XRA src`: `A = A XOR src`.
    Xra { src: Operand },
    /// `ORA src`: `A = A OR src`.
    Ora { src: Operand },
    /// `CMP src`: sets the flags from `A - src` and leaves `A` as it was.
    Cmp { src: Operand },
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
            // MVI ddd,n: 00 ddd 110
            (0b00, ddd, 0b110) => Self::Mvi {
                dst: Operand::from_code(ddd),
                value: next_byte(),
            },
            // HLT: 01 110 110, the slot where MOV M,M would be. It comes
            // before the MOV pattern, so `match` picks it first.
            (0b01, 0b110, 0b110) => Self::Hlt,
            // MOV ddd,sss: 01 ddd sss
            (0b01, ddd, sss) => Self::Mov {
                dst: Operand::from_code(ddd),
                src: Operand::from_code(sss),
            },
            // ADD sss: 10 000 sss
            (0b10, 0b000, sss) => Self::Add {
                src: Operand::from_code(sss),
            },
            // ADC sss: 10 001 sss
            (0b10, 0b001, sss) => Self::Adc {
                src: Operand::from_code(sss),
            },
            // SUB sss: 10 010 sss
            (0b10, 0b010, sss) => Self::Sub {
                src: Operand::from_code(sss),
            },
            // SBB sss: 10 011 sss
            (0b10, 0b011, sss) => Self::Sbb {
                src: Operand::from_code(sss),
            },
            // ANA sss: 10 100 sss
            (0b10, 0b100, sss) => Self::Ana {
                src: Operand::from_code(sss),
            },
            // XRA sss: 10 101 sss
            (0b10, 0b101, sss) => Self::Xra {
                src: Operand::from_code(sss),
            },
            // ORA sss: 10 110 sss
            (0b10, 0b110, sss) => Self::Ora {
                src: Operand::from_code(sss),
            },
            // CMP sss: 10 111 sss
            (0b10, 0b111, sss) => Self::Cmp {
                src: Operand::from_code(sss),
            },
            _ => unimplemented!("opcode {opcode:#04X} is not supported yet"),
        }
    }
}

/// An 8-bit operand, as opcodes encode it in a 3-bit field.
///
/// The discriminants follow the Intel manual: `B` is 000, `C` is 001 and so
/// on up to `A` at 111. `M` stands for the memory byte at the address in
/// `HL`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operand {
    B = 0,
    C = 1,
    D = 2,
    E = 3,
    H = 4,
    L = 5,
    M = 6,
    A = 7,
}

impl Operand {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operand_codes_match_intel_manual() {
        let manual_order = [
            Operand::B,
            Operand::C,
            Operand::D,
            Operand::E,
            Operand::H,
            Operand::L,
            Operand::M,
            Operand::A,
        ];

        for (code, operand) in (0..).zip(manual_order) {
            assert_eq!(Operand::from_code(code), operand, "code {code:03b}");
        }
    }

    #[test]
    fn from_code_ignores_bits_above_the_field() {
        assert_eq!(Operand::from_code(0b1111_1000), Operand::B);
        assert_eq!(Operand::from_code(0b0011_1111), Operand::A);
    }
}
