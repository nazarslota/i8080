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
//! Work in progress: only the bus traits exist so far.

#![no_std]

mod bus;

pub use bus::{IoBus, MemoryBus, Unconnected};
