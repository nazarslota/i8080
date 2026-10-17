/// One decoded 8080 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    /// `NOP`: does nothing.
    Nop,
}

impl Instruction {
    /// Decodes the instruction that starts with `opcode`.
    ///
    /// # Panics
    ///
    /// Panics on opcodes the emulator doesn't support yet. Once all 256
    /// opcodes are in, decoding can't fail.
    pub(crate) fn decode(opcode: u8) -> Self {
        match opcode {
            0x00 => Self::Nop,
            _ => unimplemented!("opcode {opcode:#04X} is not supported yet"),
        }
    }
}
