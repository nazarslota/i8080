//! Intel 8080 CPU core, independent of any machine.
//!
//! The CPU reaches memory through a [`MemoryBus`] and the I/O ports through
//! an [`IoBus`]. The machine around it implements both and lends them to
//! [`Cpu::step`] as one bus:
//!
//! ```text
//!   Cpu::step(&mut bus)
//!        │  fetch, read, write: 64 KB at 16-bit addresses     MemoryBus
//!        │  IN, OUT: 256 ports at 8-bit numbers               IoBus
//!        ▼
//!   your machine: RAM, ROM, devices
//! ```
//!
//! On the real chip both spaces share the same pins, and a status byte at
//! the start of each access tells memory and devices which space the CPU
//! means. When your memory and your devices live apart, [`Bus`] joins them.
//! So the same core runs in an Altair 8800, a CP/M test harness or an
//! arcade board.
//!
//! ```
//! use i8080::{Bus, Cpu, Unconnected};
//!
//! let mut memory = [0u8; 0x10000];
//! memory[..3].copy_from_slice(&[0x3E, 0x41, 0x76]); // MVI A,41H; HLT
//! let mut bus = Bus { memory, io: Unconnected };
//! let mut cpu = Cpu::new();
//!
//! while !cpu.is_halted() {
//!     cpu.step(&mut bus);
//! }
//!
//! assert_eq!(cpu.registers.a, 0x41);
//! ```
//!
//! The crate is `no_std` and has no dependencies, so it builds for embedded
//! targets and WebAssembly.
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
