//! MITS Altair 8800 hardware emulation, on top of the [`i8080`] CPU core.
//!
//! You plug [`cards`] into the backplane of an [`Altair`] and operate it
//! through its [`FrontPanel`], as an Altair owner did:
//!
//! ```text
//!   FrontPanel: switches, EXAMINE, DEPOSIT, RUN
//!        │
//!   Altair: i8080::Cpu ──► Backplane ──► Ram, Rom      memory
//!                                    └─► Sio2, Dcdd    I/O ports
//!                                         │     │
//!                                   Terminal   Disk    the application's side
//! ```
//!
//! The crate emulates hardware only: Altair BASIC and CP/M come from their
//! original tape and disk images. It does no host I/O either: the
//! application implements [`Terminal`](cards::Terminal) for your keyboard
//! and screen, and [`Disk`](cards::Disk) for a disk image file.
//!
//! ```
//! use altair::Altair;
//! use altair::cards::Ram;
//!
//! let mut altair = Altair::builder().plug(Ram::new(0x0000..=0xFFFF)?).build();
//! let mut panel = altair.front_panel();
//! panel.toggle_in(0x0000, &[0x3E, 0x41, 0x76]); // MVI A,41H; HLT
//! panel.set_switches(0x0000);
//! panel.examine();
//! panel.run();
//!
//! while !altair.is_halted() {
//!     altair.step();
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod cards;
mod machine;

pub use machine::{Altair, AltairBuilder, Backplane, FrontPanel};
