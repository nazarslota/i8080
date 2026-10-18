//! Intel 8080 CPU core.
//!
//! This crate emulates the processor and nothing else. The CPU reaches memory
//! through a [`MemoryBus`] and I/O ports through an [`IoBus`], and the machine
//! around it implements both. You can run the same core in an Altair 8800, a
//! CP/M test harness or an arcade board without changing it.
//!
//! The crate is `no_std` and has no dependencies, so you can build it for
//! embedded targets and WebAssembly.
//!
//! # References
//!
//! The [Intel 8080 Microcomputer Systems User's Manual] (September 1975)
//! describes every instruction with its cycle count, and we follow it. The
//! manual leaves out the undocumented copies of `NOP`, `JMP`, `CALL` and
//! `RET`. The [8080 opcode table] lists them.
//!
//! [Intel 8080 Microcomputer Systems User's Manual]: https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf
//! [8080 opcode table]: https://pastraiser.com/cpu/i8080/i8080_opcodes.html
//!
//! # Status
//!
//! The CPU runs all 256 opcodes and passes the classic diagnostic programs
//! (TST8080, 8080PRE, CPUTEST, 8080EXM). The [instruction catalog]
//! describes each instruction. Interrupts are missing: `EI` and `DI` switch
//! the interrupt enable, but nothing delivers an interrupt yet.
//!
//! [instruction catalog]: https://github.com/nazarslota/i8080/blob/main/docs/i8080/instructions.md

#![no_std]

mod alu;
mod bus;
mod condition;
mod cpu;
mod cycles;
mod flags;
mod instruction;
mod registers;

pub use bus::{Bus, IoBus, MemoryBus, Unconnected};
pub use cpu::Cpu;
pub use cycles::HALTED_STEP_CYCLES;
pub use flags::Flags;
pub use registers::Registers;
