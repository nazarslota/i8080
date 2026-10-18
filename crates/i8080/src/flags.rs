/// The five condition flags.
///
/// Arithmetic and logic instructions set the flags from their result, and
/// conditional jumps, calls and returns read them. Each flag is one bit in
/// the real 8080, and a `bool` here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// `S`: bit 7 of the result, which marks a negative number in two's
    /// complement.
    pub sign: bool,
    /// `Z`: the result is 0.
    pub zero: bool,
    /// `AC`: a carry out of bit 3 into bit 4. Only `DAA` reads it.
    pub aux_carry: bool,
    /// `P`: the result has an even number of 1 bits.
    pub parity: bool,
    /// `CY`: a carry out of bit 7 after an addition, or a borrow after a
    /// subtraction.
    pub carry: bool,
}

impl Flags {
    /// Takes `S`, `Z` and `P` from `result` and leaves `AC` and `CY` at 0.
    pub(crate) const fn from_result(result: u8) -> Self {
        Self {
            sign: result & 0x80 != 0,
            zero: result == 0,
            aux_carry: false,
            parity: result.count_ones().is_multiple_of(2),
            carry: false,
        }
    }

    /// Packs the flags into the byte that `PUSH PSW` stores.
    ///
    /// The bits follow the Intel manual: `S` is bit 7, `Z` bit 6, `AC`
    /// bit 4, `P` bit 2 and `CY` bit 0. Bit 1 is always 1, bits 5 and 3
    /// always 0.
    pub(crate) const fn to_byte(self) -> u8 {
        (self.sign as u8) << 7
            | (self.zero as u8) << 6
            | (self.aux_carry as u8) << 4
            | (self.parity as u8) << 2
            | 1 << 1
            | self.carry as u8
    }

    /// Unpacks the byte that `POP PSW` loads. It ignores bits 5, 3 and 1.
    pub(crate) const fn from_byte(byte: u8) -> Self {
        Self {
            sign: byte & 0x80 != 0,
            zero: byte & 0x40 != 0,
            aux_carry: byte & 0x10 != 0,
            parity: byte & 0x04 != 0,
            carry: byte & 0x01 != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: Flags = Flags {
        sign: true,
        zero: true,
        aux_carry: true,
        parity: true,
        carry: true,
    };

    #[test]
    fn to_byte_puts_flags_at_intel_bit_positions() {
        assert_eq!(ALL.to_byte(), 0b1101_0111);
        assert_eq!(Flags::default().to_byte(), 0b0000_0010);
        assert_eq!(
            Flags {
                aux_carry: true,
                ..Flags::default()
            }
            .to_byte(),
            0b0001_0010
        );
    }

    #[test]
    fn from_byte_ignores_unused_bits() {
        assert_eq!(Flags::from_byte(0xFF), ALL);
        assert_eq!(Flags::from_byte(0b0010_1010), Flags::default());
    }
}
