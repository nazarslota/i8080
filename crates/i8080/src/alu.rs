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

/// Adjusts `a` to two decimal digits after `ADD` or `ADC` summed two BCD
/// bytes. `aux_carry` and `carry` are the flags that addition left.
///
/// Each 4-bit half of a BCD byte holds one decimal digit, 0 to 9. A binary
/// sum can leave a half above 9 or carry out of it, and adding 6 to that
/// half skips the six values from AH to FH. `CY` stays 1 once it is 1, and
/// turns 1 when the high digit overflows.
pub(crate) const fn decimal_adjust(a: u8, aux_carry: bool, carry: bool) -> (u8, Flags) {
    let low = a & 0x0F;
    let high = a >> 4;

    let mut correction = 0;
    let mut carry_out = carry;
    if aux_carry || low > 9 {
        correction |= 0x06;
    }
    if carry || high > 9 || (high == 9 && low > 9) {
        correction |= 0x60;
        carry_out = true;
    }

    let (result, flags) = add(a, correction, false);
    (
        result,
        Flags {
            carry: carry_out,
            ..flags
        },
    )
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
