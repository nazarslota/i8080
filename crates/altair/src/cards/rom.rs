//! A PROM card. [`Rom`] documents it.

use core::error::Error;
use core::fmt;

use super::Card;

/// A PROM card: read-only memory that holds a program from its first address
/// up.
///
/// Altair owners put boot loaders into PROM, such as the disk boot loader
/// at FF00H, so the machine could start from a disk without toggling a
/// program in. A program can read the card but not write it: the card
/// ignores writes, as a PROM does.
pub struct Rom {
    start: u16,
    bytes: Vec<u8>,
}

impl Rom {
    /// Creates a card that holds `bytes` from `start` up.
    ///
    /// # Errors
    ///
    /// Returns [`RomError::Empty`] if `bytes` is empty and
    /// [`RomError::PastEndOfMemory`] if the bytes run past FFFFH.
    ///
    /// ```
    /// use altair::cards::{Card, Rom};
    ///
    /// let mut rom = Rom::new(0xFF00, vec![0xC3, 0x00, 0x00])?; // JMP 0000H
    /// rom.write(0xFF00, 0x99);
    ///
    /// assert_eq!(rom.read(0xFF00), Some(0xC3)); // the write changed nothing
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(start: u16, bytes: Vec<u8>) -> Result<Self, RomError> {
        if bytes.is_empty() {
            return Err(RomError::Empty);
        }
        if usize::from(start) + bytes.len() > usize::from(u16::MAX) + 1 {
            return Err(RomError::PastEndOfMemory);
        }
        Ok(Self { start, bytes })
    }

    /// Returns the index in `bytes` for `addr`, or `None` if the card doesn't
    /// claim `addr`.
    fn index(&self, addr: u16) -> Option<usize> {
        let index = usize::from(addr.checked_sub(self.start)?);
        (index < self.bytes.len()).then_some(index)
    }
}

impl Card for Rom {
    fn read(&mut self, addr: u16) -> Option<u8> {
        self.index(addr).map(|index| self.bytes[index])
    }
}

/// Why [`Rom::new`] can't make the card.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RomError {
    /// You gave no bytes.
    Empty,
    /// The bytes run past FFFFH, the last address of the 8080.
    PastEndOfMemory,
}

impl fmt::Display for RomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::Empty => "a PROM card needs at least one byte",
            Self::PastEndOfMemory => "the PROM runs past FFFFH",
        };
        f.write_str(message)
    }
}

impl Error for RomError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_bytes_from_start_up() {
        let mut rom = Rom::new(0xFF00, vec![0x11, 0x22]).unwrap();

        let bytes = [rom.read(0xFF00), rom.read(0xFF01)];

        assert_eq!(bytes, [Some(0x11), Some(0x22)]);
    }

    #[test]
    fn ignores_writes() {
        let mut rom = Rom::new(0xFF00, vec![0x11]).unwrap();

        rom.write(0xFF00, 0x99);

        assert_eq!(rom.read(0xFF00), Some(0x11));
    }

    #[test]
    fn does_not_claim_addresses_outside() {
        let mut rom = Rom::new(0xFF00, vec![0x11, 0x22]).unwrap();

        let outside = [rom.read(0xFEFF), rom.read(0xFF02)];

        assert_eq!(outside, [None, None]);
    }

    #[test]
    fn fills_memory_up_to_ffffh() {
        let mut rom = Rom::new(0xFF00, vec![0x33; 0x100]).unwrap();

        let last = rom.read(0xFFFF);

        assert_eq!(last, Some(0x33));
    }

    #[test]
    fn new_rejects_empty_bytes() {
        let rom = Rom::new(0xFF00, Vec::new());

        assert_eq!(rom.err(), Some(RomError::Empty));
    }

    #[test]
    fn new_rejects_bytes_past_ffffh() {
        let rom = Rom::new(0xFF00, vec![0x00; 0x101]);

        assert_eq!(rom.err(), Some(RomError::PastEndOfMemory));
    }
}
