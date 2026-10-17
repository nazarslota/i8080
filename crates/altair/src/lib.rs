//! MITS Altair 8800 hardware emulation.
//!
//! The Altair is modelled the way the real machine was built: an S-100
//! backplane with expansion cards plugged into it (memory, the 88-2SIO serial
//! board, the 88-DCDD floppy controller). Each card decides which memory
//! addresses and I/O ports it answers.
//!
//! This crate emulates hardware only. Software such as Altair BASIC, Altair
//! DOS or CP/M is not part of it: it is loaded from ROM, tape or disk images,
//! just like on the original machine. The crate does no host I/O either; the
//! application connects the serial card to a real terminal.
//!
//! # Status
//!
//! Work in progress: the public API is not defined yet.
