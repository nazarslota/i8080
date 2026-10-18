//! The cards you plug into the Altair's backplane.
//!
//! The CPU card comes with every [`Altair`](crate::Altair). Each other card
//! implements [`Card`] and answers the addresses or ports it claims:
//!
//! | Card     | Claims                               | Plays the role of                         |
//! |----------|--------------------------------------|-------------------------------------------|
//! | [`Ram`]  | the addresses you give it            | a memory card                             |
//! | [`Rom`]  | its bytes, from the address you give | a PROM card, such as the disk boot loader |
//! | [`Sio2`] | ports `10H` and `11H`                | the 88-2SIO serial card, port A           |
//! | [`Dcdd`] | ports `08H` to `0AH`                 | the 88-DCDD floppy disk controller        |
//!
//! ```
//! use altair::cards::{Card, Ram};
//!
//! let mut ram = Ram::new(0x0000..=0x00FF)?;
//!
//! assert_eq!(ram.read(0x0010), Some(0x00)); // the card claims 0010H
//! assert_eq!(ram.read(0x0100), None); // and stays silent at 0100H
//! assert_eq!(ram.input(0x10), None); // a memory card claims no port
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod dcdd;
mod ram;
mod rom;
mod sio2;

pub use dcdd::{DISK_BYTES, Dcdd, Disk, NoSuchDriveError};
pub use ram::{EmptyRangeError, Ram};
pub use rom::{Rom, RomError};
pub use sio2::{Sio2, Terminal};

/// A card that plugs into the [`Backplane`](crate::Backplane).
///
/// A real S-100 card sees every bus cycle. Its address decoder checks the
/// address and either answers or stays silent. `Card` works the same way:
/// the backplane passes every access to every card, and each card returns
/// `None` from a read it doesn't claim and ignores such a write.
///
/// Every method has a default that claims nothing, so a card implements only
/// the address space it uses: a memory card implements `read` and `write`, a
/// serial card `input` and `output`.
pub trait Card {
    /// Returns the byte at `addr`, or `None` if the card doesn't claim the
    /// address.
    fn read(&mut self, _addr: u16) -> Option<u8> {
        None
    }

    /// Writes `value` to `addr` if the card claims the address.
    fn write(&mut self, _addr: u16, _value: u8) {}

    /// Returns the byte from `port`, or `None` if the card doesn't claim the
    /// port.
    fn input(&mut self, _port: u8) -> Option<u8> {
        None
    }

    /// Writes `value` to `port` if the card claims the port.
    fn output(&mut self, _port: u8, _value: u8) {}
}
