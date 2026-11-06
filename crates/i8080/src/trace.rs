//! The execution trace. [`TraceLine`] documents it.

use core::fmt::{self, Write};

use crate::disassembler::{Disassembled, MAX_INSTRUCTION_BYTES, disassemble};
use crate::{Flags, Registers};

/// The width of the instruction column: the longest instruction,
/// `LXI SP,0FFFFH`, and two spaces.
const INSTRUCTION_COLUMN_BYTES: usize = 15;

/// One line of an execution trace: the instruction at `PC` and the
/// registers before the CPU runs it.
///
/// [`Display`](fmt::Display) writes the line in columns under
/// [`HEADER`](Self::HEADER):
///
/// ```text
/// PC    BYTES     INSTRUCTION    A  BC   DE   HL   SP   SZAPC
/// FF00  21 13 FF  LXI H,0FF13H   00 0000 0000 0000 0000 .....
/// FF03  11 00 2C  LXI D,2C00H    00 0000 0000 FF13 0000 .....
/// ```
///
/// `SZAPC` shows the flags: sign, zero, auxiliary carry, parity and carry,
/// with a letter for a set flag and a dot for a clear one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceLine {
    registers: Registers,
    instruction: Disassembled,
}

impl TraceLine {
    /// The column names, to print above the lines.
    pub const HEADER: &str = "PC    BYTES     INSTRUCTION    A  BC   DE   HL   SP   SZAPC";

    /// Decodes the instruction at `registers.pc` and keeps it with the
    /// registers.
    ///
    /// `read` returns the byte at an address. The function calls it for each
    /// byte of the instruction, up to three. Read memory without touching a
    /// device, so the trace doesn't change what the program sees.
    ///
    /// ```
    /// use i8080::{Registers, TraceLine};
    ///
    /// let memory = [0x21, 0x13, 0xFF]; // LXI H,0FF13H
    /// let registers = Registers::default();
    ///
    /// let line = TraceLine::new(registers, |address| memory[usize::from(address)]);
    ///
    /// assert_eq!(
    ///     line.to_string(),
    ///     "0000  21 13 FF  LXI H,0FF13H   00 0000 0000 0000 0000 .....",
    /// );
    /// ```
    pub fn new(registers: Registers, mut read: impl FnMut(u16) -> u8) -> Self {
        let mut address = registers.pc;
        let instruction = disassemble(|| {
            let byte = read(address);
            address = address.wrapping_add(1);
            byte
        });

        Self {
            registers,
            instruction,
        }
    }
}

impl fmt::Display for TraceLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let registers = &self.registers;
        write!(f, "{:04X}  ", registers.pc)?;
        let bytes = self.instruction.bytes();
        for byte in bytes {
            write!(f, "{byte:02X} ")?;
        }
        let padding = (MAX_INSTRUCTION_BYTES - bytes.len()) * 3 + 1;
        write!(f, "{:padding$}", "")?;

        let mut column = Counter { f, bytes: 0 };
        write!(column, "{}", self.instruction)?;
        let padding = INSTRUCTION_COLUMN_BYTES.saturating_sub(column.bytes);
        write!(f, "{:padding$}", "")?;

        write!(
            f,
            "{:02X} {:04X} {:04X} {:04X} {:04X} {}",
            registers.a,
            registers.bc(),
            registers.de(),
            registers.hl(),
            registers.sp,
            FlagLetters(registers.flags),
        )
    }
}

/// Passes text on to a formatter and counts its bytes, so the caller can pad
/// a column. A `no_std` crate has no `String` to measure first.
struct Counter<'a, 'b> {
    f: &'a mut fmt::Formatter<'b>,
    bytes: usize,
}

impl Write for Counter<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.bytes += text.len();
        self.f.write_str(text)
    }
}

/// Writes the flags as `SZAPC`, with a dot for each clear flag.
struct FlagLetters(Flags);

impl fmt::Display for FlagLetters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = self.0;
        for (set, letter) in [
            (flags.sign, 'S'),
            (flags.zero, 'Z'),
            (flags.aux_carry, 'A'),
            (flags.parity, 'P'),
            (flags.carry, 'C'),
        ] {
            f.write_char(if set { letter } else { '.' })?;
        }
        Ok(())
    }
}
