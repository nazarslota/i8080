//! The cards you plug into the Altair's backplane.
//!
//! The Altair needs the CPU card ([`Cpu`]), a memory card ([`Ram`]) and a
//! serial card for the terminal ([`Sio2`]). The floppy disk controller
//! ([`Dcdd`]) reads and writes disks, and a PROM card ([`Rom`]) holds the
//! loader that boots from them. Each card except the CPU implements
//! [`Card`].

mod cpu;
mod dcdd;
mod ram;
mod rom;
mod sio2;

pub use cpu::Cpu;
pub use dcdd::{DISK_BYTES, Dcdd, NoSuchDriveError};
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
