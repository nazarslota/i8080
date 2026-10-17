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
//! describes every instruction with its cycle count, and we follow it.
//!
//! [Intel 8080 Microcomputer Systems User's Manual]: https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf
//!
//! # Status
//!
//! Work in progress: the CPU runs `NOP`, `MVI`, `MOV`, `HLT`, `ADD`, `ADC`,
//! `SUB` and `SBB` and panics on every other opcode.

#![no_std]

mod bus;
mod cpu;
mod cycles;
mod flags;
mod instruction;
mod registers;

pub use bus::{IoBus, MemoryBus, Unconnected};
pub use cpu::Cpu;
pub use cycles::HALTED_STEP_CYCLES;
pub use flags::Flags;
pub use registers::Registers;
