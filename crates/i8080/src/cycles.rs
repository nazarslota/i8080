//! Clock cycles per instruction.
//!
//! The counts come from the Intel 8080 Microcomputer Systems User's Manual
//! (September 1975):
//! <https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf>
//!
//! An instruction that touches memory through `M` takes longer, because each
//! memory access costs 3 cycles on the bus. Those variants get their own
//! constant with an `_M` suffix.

/// `NOP`.
pub(crate) const NOP: u32 = 4;

/// `LXI rp,nn`: opcode fetch plus two reads for the constant.
pub(crate) const LXI: u32 = 10;

/// `DAD rp`.
pub(crate) const DAD: u32 = 10;

/// `INX rp`.
pub(crate) const INX: u32 = 5;

/// `DCX rp`.
pub(crate) const DCX: u32 = 5;

/// `INR r`.
pub(crate) const INR: u32 = 5;
/// `INR M`: `INR` plus a memory read and a memory write.
pub(crate) const INR_M: u32 = 10;

/// `DCR r`.
pub(crate) const DCR: u32 = 5;
/// `DCR M`: `DCR` plus a memory read and a memory write.
pub(crate) const DCR_M: u32 = 10;

/// `MVI r,n`: opcode fetch plus one read for the constant.
pub(crate) const MVI: u32 = 7;
/// `MVI M,n`: `MVI` plus a write to memory.
pub(crate) const MVI_M: u32 = 10;

/// `HLT`.
pub(crate) const HLT: u32 = 7;

/// `MOV r,r`.
pub(crate) const MOV: u32 = 5;
/// `MOV r,M` and `MOV M,r`: a memory read or write replaces the register
/// transfer.
pub(crate) const MOV_M: u32 = 7;

/// `ADD r`.
pub(crate) const ADD: u32 = 4;
/// `ADD M`: `ADD` plus a memory read.
pub(crate) const ADD_M: u32 = 7;

/// `ADC r`.
pub(crate) const ADC: u32 = 4;
/// `ADC M`: `ADC` plus a memory read.
pub(crate) const ADC_M: u32 = 7;

/// `SUB r`.
pub(crate) const SUB: u32 = 4;
/// `SUB M`: `SUB` plus a memory read.
pub(crate) const SUB_M: u32 = 7;

/// `SBB r`.
pub(crate) const SBB: u32 = 4;
/// `SBB M`: `SBB` plus a memory read.
pub(crate) const SBB_M: u32 = 7;

/// `ANA r`.
pub(crate) const ANA: u32 = 4;
/// `ANA M`: `ANA` plus a memory read.
pub(crate) const ANA_M: u32 = 7;

/// `XRA r`.
pub(crate) const XRA: u32 = 4;
/// `XRA M`: `XRA` plus a memory read.
pub(crate) const XRA_M: u32 = 7;

/// `ORA r`.
pub(crate) const ORA: u32 = 4;
/// `ORA M`: `ORA` plus a memory read.
pub(crate) const ORA_M: u32 = 7;

/// `CMP r`.
pub(crate) const CMP: u32 = 4;
/// `CMP M`: `CMP` plus a memory read.
pub(crate) const CMP_M: u32 = 7;

/// `ADI n`: opcode fetch plus one read for the constant.
pub(crate) const ADI: u32 = 7;

/// `ACI n`: opcode fetch plus one read for the constant.
pub(crate) const ACI: u32 = 7;

/// `SUI n`: opcode fetch plus one read for the constant.
pub(crate) const SUI: u32 = 7;

/// `SBI n`: opcode fetch plus one read for the constant.
pub(crate) const SBI: u32 = 7;

/// `ANI n`: opcode fetch plus one read for the constant.
pub(crate) const ANI: u32 = 7;

/// `XRI n`: opcode fetch plus one read for the constant.
pub(crate) const XRI: u32 = 7;

/// `ORI n`: opcode fetch plus one read for the constant.
pub(crate) const ORI: u32 = 7;

/// `CPI n`: opcode fetch plus one read for the constant.
pub(crate) const CPI: u32 = 7;

/// Clock cycles that [`Cpu::step`](crate::Cpu::step) reports while the CPU is
/// halted.
///
/// A halted 8080 does nothing until an interrupt arrives, so the count is our
/// choice rather than a number from the manual. Reporting the length of a
/// `NOP` keeps the machine's clock moving at a steady pace.
pub const HALTED_STEP_CYCLES: u32 = NOP;
