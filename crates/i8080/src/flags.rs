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
            parity: result.count_ones() % 2 == 0,
            carry: false,
        }
    }
}
