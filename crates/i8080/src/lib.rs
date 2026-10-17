//! Intel 8080 CPU core.
//!
//! This crate emulates the processor and nothing else. It has no idea what
//! machine it runs in: memory and I/O ports are reached only through a bus
//! that the host machine provides. The same core can therefore power an
//! Altair 8800, a CP/M test harness or an arcade board without changes.
//!
//! The crate is `no_std` and has no dependencies, so it also builds for
//! embedded targets and WebAssembly.
//!
//! # Status
//!
//! Work in progress: the public API is not defined yet.

#![no_std]
