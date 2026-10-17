use crate::Flags;

/// The 8080 registers that programs can see, flags included.
///
/// The fields are public: any value is valid in any register, so code that
/// drives the CPU (tests, a debugger, the Altair front panel) can read and set
/// them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// The accumulator. Most arithmetic and logic instructions put their
    /// result here.
    pub a: u8,
    /// Register B, the high byte of the `BC` pair.
    pub b: u8,
    /// Register C, the low byte of the `BC` pair.
    pub c: u8,
    /// Register D, the high byte of the `DE` pair.
    pub d: u8,
    /// Register E, the low byte of the `DE` pair.
    pub e: u8,
    /// Register H, the high byte of the `HL` pair.
    pub h: u8,
    /// Register L, the low byte of the `HL` pair.
    pub l: u8,
    /// The stack pointer: the address of the top of the stack.
    pub sp: u16,
    /// The program counter: the address of the next instruction to fetch.
    pub pc: u16,
    /// The condition flags.
    pub flags: Flags,
}

/// Register pairs. Each pair joins two 8-bit registers into one 16-bit value,
/// with the first register as the high byte: in `BC`, `B` is high and `C` is
/// low.
impl Registers {
    /// The `BC` pair.
    pub const fn bc(&self) -> u16 {
        u16::from_be_bytes([self.b, self.c])
    }

    /// Sets the `BC` pair: the high byte goes to `B`, the low byte to `C`.
    pub const fn set_bc(&mut self, value: u16) {
        [self.b, self.c] = value.to_be_bytes();
    }

    /// The `DE` pair.
    pub const fn de(&self) -> u16 {
        u16::from_be_bytes([self.d, self.e])
    }

    /// Sets the `DE` pair: the high byte goes to `D`, the low byte to `E`.
    pub const fn set_de(&mut self, value: u16) {
        [self.d, self.e] = value.to_be_bytes();
    }

    /// The `HL` pair.
    pub const fn hl(&self) -> u16 {
        u16::from_be_bytes([self.h, self.l])
    }

    /// Sets the `HL` pair: the high byte goes to `H`, the low byte to `L`.
    pub const fn set_hl(&mut self, value: u16) {
        [self.h, self.l] = value.to_be_bytes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_read_first_register_as_high_byte() {
        let registers = Registers {
            b: 0x12,
            c: 0x34,
            d: 0x56,
            e: 0x78,
            h: 0x9A,
            l: 0xBC,
            ..Registers::default()
        };

        let pairs = (registers.bc(), registers.de(), registers.hl());

        assert_eq!(pairs, (0x1234, 0x5678, 0x9ABC));
    }

    #[test]
    fn set_pair_writes_high_byte_to_first_register() {
        let mut registers = Registers::default();

        registers.set_bc(0x1234);
        registers.set_de(0x5678);
        registers.set_hl(0x9ABC);

        assert_eq!((registers.b, registers.c), (0x12, 0x34));
        assert_eq!((registers.d, registers.e), (0x56, 0x78));
        assert_eq!((registers.h, registers.l), (0x9A, 0xBC));
    }
}
