//! The arithmetic and logic unit: pure functions that take `A` and an
//! operand and return the result with the flags it sets.

use crate::Flags;

/// Adds `value` and `carry_in` to `a`.
pub(crate) const fn add(a: u8, value: u8, carry_in: bool) -> (u8, Flags) {
    let carry_in = carry_in as u8;
    // A u16 holds the full sum, so bit 8 is the carry out of bit 7.
    let sum = a as u16 + value as u16 + carry_in as u16;
    let [carry_out, result] = sum.to_be_bytes();

    let flags = Flags {
        aux_carry: (a & 0x0F) + (value & 0x0F) + carry_in > 0x0F,
        carry: carry_out != 0,
        ..Flags::from_result(result)
    };
    (result, flags)
}

/// Subtracts `value` and `borrow_in` from `a`.
///
/// The 8080 subtracts with its adder: `a - value` equals `a + !value + 1`.
/// `AC` comes out of that addition as it is, and `CY` comes out inverted, so
/// `CY = 1` means the subtraction borrowed.
pub(crate) const fn subtract(a: u8, value: u8, borrow_in: bool) -> (u8, Flags) {
    let (result, mut flags) = add(a, !value, !borrow_in);
    flags.carry = !flags.carry;
    (result, flags)
}

/// `a AND value`. Clears `CY`.
///
/// The 8080 sets `AC` to bit 3 of `a OR value`. The Intel 8085 sets it to 1
/// instead, and CPU diagnostics such as 8080EXM check the 8080 behavior.
pub(crate) const fn and(a: u8, value: u8) -> (u8, Flags) {
    let result = a & value;
    let flags = Flags {
        aux_carry: (a | value) & 0x08 != 0,
        ..Flags::from_result(result)
    };
    (result, flags)
}

/// `a XOR value`. Clears `AC` and `CY`.
pub(crate) const fn xor(a: u8, value: u8) -> (u8, Flags) {
    let result = a ^ value;
    (result, Flags::from_result(result))
}

/// `a OR value`. Clears `AC` and `CY`.
pub(crate) const fn or(a: u8, value: u8) -> (u8, Flags) {
    let result = a | value;
    (result, Flags::from_result(result))
}
