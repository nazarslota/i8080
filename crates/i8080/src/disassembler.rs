//! The disassembler. [`disassemble`] documents it.

use core::fmt;

use crate::condition::Condition;
use crate::instruction::{Instruction, Operand8, Operand16, StackOperand};

/// The most bytes one instruction takes: an opcode and a 16-bit constant.
pub(crate) const MAX_INSTRUCTION_BYTES: usize = 3;

/// One instruction, ready to print. [`disassemble`] makes it.
///
/// [`Display`](fmt::Display) writes Intel assembler: `JMP 0C000H`. A hex
/// number starts with a digit, so a value such as `0FFH` keeps a leading 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Disassembled {
    instruction: Instruction,
    bytes: [u8; MAX_INSTRUCTION_BYTES],
    length: usize,
}

impl Disassembled {
    /// The instruction's bytes, opcode first. Their count tells you where the
    /// next instruction starts. See [`disassemble`] for an example.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

impl fmt::Display for Disassembled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.instruction.fmt(f)
    }
}

/// Decodes one instruction for printing, with the decoder [`Cpu`](crate::Cpu)
/// runs.
///
/// `next_byte` returns the instruction's bytes in order, opcode first. The
/// function calls it once for each byte the instruction has, up to three.
/// Reading through a closure lets you choose the source: a slice, or memory
/// that you read without touching a device.
///
/// ```
/// let mut bytes = [0xC3, 0x00, 0xC0, 0x76].into_iter();
///
/// let jump = i8080::disassemble(|| bytes.next().unwrap_or(0));
/// let halt = i8080::disassemble(|| bytes.next().unwrap_or(0));
///
/// assert_eq!(jump.to_string(), "JMP 0C000H");
/// assert_eq!(jump.bytes(), [0xC3, 0x00, 0xC0]);
/// assert_eq!(halt.to_string(), "HLT");
/// assert_eq!(halt.bytes(), [0x76]);
/// ```
///
/// The 12 undocumented opcodes print as the instructions they copy, because
/// the CPU runs them that way: `CBH` prints `JMP`. [`bytes`](Disassembled::bytes)
/// keeps the original opcode.
pub fn disassemble(mut next_byte: impl FnMut() -> u8) -> Disassembled {
    let mut bytes = [0; MAX_INSTRUCTION_BYTES];
    let mut length = 0;
    let instruction = Instruction::decode(|| {
        let byte = next_byte();
        bytes[length] = byte; // the decoder asks for three bytes at most
        length += 1;
        byte
    });

    Disassembled {
        instruction,
        bytes,
        length,
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Nop => f.write_str("NOP"),
            Self::Lxi { dst, value } => write!(f, "LXI {dst},{}", Word(value)),
            Self::Stax { dst } => write!(f, "STAX {dst}"),
            Self::Inx { pair } => write!(f, "INX {pair}"),
            Self::Inr { dst } => write!(f, "INR {dst}"),
            Self::Dcr { dst } => write!(f, "DCR {dst}"),
            Self::Mvi { dst, value } => write!(f, "MVI {dst},{}", Byte(value)),
            Self::Rlc => f.write_str("RLC"),
            Self::Dad { src } => write!(f, "DAD {src}"),
            Self::Ldax { src } => write!(f, "LDAX {src}"),
            Self::Dcx { pair } => write!(f, "DCX {pair}"),
            Self::Rrc => f.write_str("RRC"),
            Self::Ral => f.write_str("RAL"),
            Self::Rar => f.write_str("RAR"),
            Self::Shld { address } => write!(f, "SHLD {}", Word(address)),
            Self::Daa => f.write_str("DAA"),
            Self::Lhld { address } => write!(f, "LHLD {}", Word(address)),
            Self::Cma => f.write_str("CMA"),
            Self::Sta { address } => write!(f, "STA {}", Word(address)),
            Self::Stc => f.write_str("STC"),
            Self::Lda { address } => write!(f, "LDA {}", Word(address)),
            Self::Cmc => f.write_str("CMC"),
            Self::Hlt => f.write_str("HLT"),
            Self::Mov { dst, src } => write!(f, "MOV {dst},{src}"),
            Self::Add { src } => write!(f, "ADD {src}"),
            Self::Adc { src } => write!(f, "ADC {src}"),
            Self::Sub { src } => write!(f, "SUB {src}"),
            Self::Sbb { src } => write!(f, "SBB {src}"),
            Self::Ana { src } => write!(f, "ANA {src}"),
            Self::Xra { src } => write!(f, "XRA {src}"),
            Self::Ora { src } => write!(f, "ORA {src}"),
            Self::Cmp { src } => write!(f, "CMP {src}"),
            Self::Rcc { condition } => write!(f, "R{condition}"),
            Self::Pop { dst } => write!(f, "POP {dst}"),
            Self::Jcc { condition, address } => write!(f, "J{condition} {}", Word(address)),
            Self::Jmp { address } => write!(f, "JMP {}", Word(address)),
            Self::Ccc { condition, address } => write!(f, "C{condition} {}", Word(address)),
            Self::Push { src } => write!(f, "PUSH {src}"),
            Self::Adi { value } => write!(f, "ADI {}", Byte(value)),
            // RST n calls 8 × n.
            Self::Rst { address } => write!(f, "RST {}", address / 8),
            Self::Ret => f.write_str("RET"),
            Self::Call { address } => write!(f, "CALL {}", Word(address)),
            Self::Aci { value } => write!(f, "ACI {}", Byte(value)),
            Self::Out { port } => write!(f, "OUT {}", Byte(port)),
            Self::Sui { value } => write!(f, "SUI {}", Byte(value)),
            Self::In { port } => write!(f, "IN {}", Byte(port)),
            Self::Sbi { value } => write!(f, "SBI {}", Byte(value)),
            Self::Xthl => f.write_str("XTHL"),
            Self::Ani { value } => write!(f, "ANI {}", Byte(value)),
            Self::Pchl => f.write_str("PCHL"),
            Self::Xchg => f.write_str("XCHG"),
            Self::Xri { value } => write!(f, "XRI {}", Byte(value)),
            Self::Di => f.write_str("DI"),
            Self::Ori { value } => write!(f, "ORI {}", Byte(value)),
            Self::Sphl => f.write_str("SPHL"),
            Self::Ei => f.write_str("EI"),
            Self::Cpi { value } => write!(f, "CPI {}", Byte(value)),
        }
    }
}

/// The variants spell the Intel names.
impl fmt::Display for Operand8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Intel assembler names a pair by its first register.
impl fmt::Display for Operand16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BC => "B",
            Self::DE => "D",
            Self::HL => "H",
            Self::SP => "SP",
        })
    }
}

/// Intel assembler names a pair by its first register.
impl fmt::Display for StackOperand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BC => "B",
            Self::DE => "D",
            Self::HL => "H",
            Self::PSW => "PSW",
        })
    }
}

/// The variants spell the Intel suffixes.
impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// An 8-bit constant or port in Intel notation: `41H`, `0FFH`.
struct Byte(u8);

impl fmt::Display for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 0xA0 {
            f.write_str("0")?;
        }
        write!(f, "{:02X}H", self.0)
    }
}

/// A 16-bit constant or address in Intel notation: `2000H`, `0C000H`.
struct Word(u16);

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 0xA000 {
            f.write_str("0")?;
        }
        write!(f, "{:04X}H", self.0)
    }
}
