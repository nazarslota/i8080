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

/// `MVI r,n`: opcode fetch plus one read for the constant.
pub(crate) const MVI: u32 = 7;
/// `MVI M,n`: `MVI` plus a write to memory.
pub(crate) const MVI_M: u32 = 10;

/// `MOV r,r`.
pub(crate) const MOV: u32 = 5;
/// `MOV r,M` and `MOV M,r`: a memory read or write replaces the register
/// transfer.
pub(crate) const MOV_M: u32 = 7;

/// `ADD r`.
pub(crate) const ADD: u32 = 4;
/// `ADD M`: `ADD` plus a memory read.
pub(crate) const ADD_M: u32 = 7;

/// `HLT`.
pub(crate) const HLT: u32 = 7;

/// Clock cycles that [`Cpu::step`](crate::Cpu::step) reports while the CPU is
/// halted.
///
/// A halted 8080 does nothing until an interrupt arrives, so the count is our
/// choice rather than a number from the manual. Reporting the length of a
/// `NOP` keeps the machine's clock moving at a steady pace.
pub const HALTED_STEP_CYCLES: u32 = NOP;
