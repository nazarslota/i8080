//! MITS Altair 8800 hardware emulation.
//!
//! The crate models the Altair the way MITS built it: an S-100 backplane with
//! expansion cards plugged into it (memory, the 88-2SIO serial board, the
//! 88-DCDD floppy controller). Each card decides which memory addresses and
//! I/O ports it answers.
//!
//! This crate emulates hardware only. Altair BASIC, Altair DOS and CP/M come
//! from ROM, tape or disk images, as on the original machine. The crate does
//! no host I/O either; the application connects the serial card to a real
//! terminal.
//!
//! # Status
//!
//! Work in progress: the [`Backplane`] and the [`Ram`] card work. The serial
//! card comes next.

mod backplane;
mod device;
mod ram;

pub use backplane::Backplane;
pub use device::Device;
pub use ram::Ram;
