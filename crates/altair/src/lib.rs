//! MITS Altair 8800 hardware emulation.
//!
//! The crate models the Altair the way MITS built it: an S-100 backplane with
//! [`cards`] plugged into it (memory, the 88-2SIO serial board, the
//! 88-DCDD floppy controller), and a [`FrontPanel`] with switches. Each card
//! decides which memory addresses and I/O ports it answers. You pick the
//! cards with [`Altair::builder`].
//!
//! This crate emulates hardware only. Altair BASIC, Altair DOS and CP/M come
//! from ROM, tape or disk images, as on the original machine. The crate does
//! no host I/O either; the application connects the serial card to a real
//! terminal.
//!
//! # Status
//!
//! Work in progress: the [`Altair`] runs programs that you toggle in with its
//! [`FrontPanel`], and Altair BASIC from its paper tape. The 88-DCDD floppy
//! controller comes next.

pub mod cards;
mod machine;

pub use machine::{Altair, AltairBuilder, Backplane, FrontPanel, Plug};
