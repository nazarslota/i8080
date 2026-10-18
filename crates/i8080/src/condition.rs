//! The conditions of the conditional jumps, calls and returns: the `CCC`
//! field of their opcodes.
//!
//! ```text
//!   JNZ addr   11 000 010   CCC = 000: NZ, jumps when Z = 0
//!   JZ addr    11 001 010   CCC = 001: Z,  jumps when Z = 1
//!   JM addr    11 111 010   CCC = 111: M,  jumps when S = 1
//! ```

use crate::Flags;

/// A condition for a conditional jump, call or return, as opcodes encode
/// it in a 3-bit field.
///
/// The discriminants follow the Intel manual: `NZ` is 000 and so on up to
/// `M` at 111. Each condition tests one flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Condition {
    /// Not zero: `Z = 0`.
    NZ = 0,
    /// Zero: `Z = 1`.
    Z = 1,
    /// No carry: `CY = 0`.
    NC = 2,
    /// Carry: `CY = 1`.
    C = 3,
    /// Parity odd: `P = 0`.
    PO = 4,
    /// Parity even: `P = 1`.
    PE = 5,
    /// Plus: `S = 0`.
    P = 6,
    /// Minus: `S = 1`.
    M = 7,
}

impl Condition {
    /// Decodes a 3-bit condition field. It ignores bits above the lowest
    /// three.
    pub(crate) const fn from_code(code: u8) -> Self {
        match code & 0b111 {
            0 => Self::NZ,
            1 => Self::Z,
            2 => Self::NC,
            3 => Self::C,
            4 => Self::PO,
            5 => Self::PE,
            6 => Self::P,
            _ => Self::M,
        }
    }

    /// Tests the condition against the flags.
    pub(crate) const fn holds(self, flags: Flags) -> bool {
        match self {
            Self::NZ => !flags.zero,
            Self::Z => flags.zero,
            Self::NC => !flags.carry,
            Self::C => flags.carry,
            Self::PO => !flags.parity,
            Self::PE => flags.parity,
            Self::P => !flags.sign,
            Self::M => flags.sign,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_codes_match_intel_manual() {
        let manual_order = [
            Condition::NZ,
            Condition::Z,
            Condition::NC,
            Condition::C,
            Condition::PO,
            Condition::PE,
            Condition::P,
            Condition::M,
        ];

        for (code, condition) in (0..).zip(manual_order) {
            assert_eq!(Condition::from_code(code), condition, "code {code:03b}");
        }
    }

    #[test]
    fn holds_tests_one_flag() {
        // Each case: a condition, the flags and whether the condition holds.
        // Each condition gets one case with its flag at 0 and one at 1.
        let none = Flags::default();
        let cases = [
            (Condition::NZ, none, true),
            (Condition::NZ, Flags { zero: true, ..none }, false),
            (Condition::Z, none, false),
            (Condition::Z, Flags { zero: true, ..none }, true),
            (Condition::NC, none, true),
            (
                Condition::NC,
                Flags {
                    carry: true,
                    ..none
                },
                false,
            ),
            (Condition::C, none, false),
            (
                Condition::C,
                Flags {
                    carry: true,
                    ..none
                },
                true,
            ),
            (Condition::PO, none, true),
            (
                Condition::PO,
                Flags {
                    parity: true,
                    ..none
                },
                false,
            ),
            (Condition::PE, none, false),
            (
                Condition::PE,
                Flags {
                    parity: true,
                    ..none
                },
                true,
            ),
            (Condition::P, none, true),
            (Condition::P, Flags { sign: true, ..none }, false),
            (Condition::M, none, false),
            (Condition::M, Flags { sign: true, ..none }, true),
        ];

        for (condition, flags, expected) in cases {
            assert_eq!(
                condition.holds(flags),
                expected,
                "{condition:?} with {flags:?}"
            );
        }
    }
}
