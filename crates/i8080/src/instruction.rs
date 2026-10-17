/// One decoded 8080 instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    /// `NOP`: does nothing.
    Nop,
    /// `MVI B,value`: loads the byte that follows the opcode into B.
    MviB(u8),
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
        match opcode {
            0x00 => Self::Nop,
            0x06 => Self::MviB(next_byte()),
            _ => unimplemented!("opcode {opcode:#04X} is not supported yet"),
        }
    }
}
