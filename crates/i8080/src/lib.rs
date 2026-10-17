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
//! # Status
//!
//! Work in progress: the bus traits and the registers exist so far.

#![no_std]

mod bus;
mod registers;

pub use bus::{IoBus, MemoryBus, Unconnected};
pub use registers::Registers;
