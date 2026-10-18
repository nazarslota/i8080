//! A memory card. [`Ram`] documents it.

use core::error::Error;
use core::fmt;
use core::ops::RangeInclusive;

use super::Card;

/// A memory card: RAM that claims one range of addresses.
///
/// The Altair shipped with 256 bytes of RAM, and owners added memory cards.
/// You pick the range of addresses when you create the card.
pub struct Ram {
    start: u16,
    bytes: Vec<u8>,
}

impl Ram {
    /// Creates a card that claims `addresses`, with every byte at 0.
    ///
    /// Real RAM powers up with random bytes. Zeros make every run of a
    /// program start from the same state.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyRangeError`] if `addresses` is empty, such as
    /// `0x2000..=0x1000`.
    ///
    /// ```
    /// use altair::cards::{Card, Ram};
    ///
    /// let mut ram = Ram::new(0x2000..=0x2FFF)?; // 4 KB at 2000H
    /// ram.write(0x2000, 0x99);
    ///
    /// assert_eq!(ram.read(0x2000), Some(0x99));
    /// assert_eq!(ram.read(0x1FFF), None); // below the card
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(addresses: RangeInclusive<u16>) -> Result<Self, EmptyRangeError> {
        let (start, end) = addresses.into_inner();
        if start > end {
            return Err(EmptyRangeError);
        }
        let size = usize::from(end - start) + 1;
        Ok(Self {
            start,
            bytes: vec![0; size],
        })
    }

    /// Returns the index in `bytes` for `addr`, or `None` if the card doesn't
    /// claim `addr`.
    fn index(&self, addr: u16) -> Option<usize> {
        let index = usize::from(addr.checked_sub(self.start)?);
        (index < self.bytes.len()).then_some(index)
    }
}

impl Card for Ram {
    fn read(&mut self, addr: u16) -> Option<u8> {
        self.index(addr).map(|index| self.bytes[index])
    }

    fn write(&mut self, addr: u16, value: u8) {
        if let Some(index) = self.index(addr) {
            self.bytes[index] = value;
        }
    }
}

/// The range you gave [`Ram::new`] holds no address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmptyRangeError;

impl fmt::Display for EmptyRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a memory card needs at least one address")
    }
}

impl Error for EmptyRangeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fills_memory_with_zeros() {
        let mut ram = Ram::new(0x1000..=0x1FFF).unwrap();

        let value = ram.read(0x1234);

        assert_eq!(value, Some(0x00));
    }

    #[test]
    #[allow(
        clippy::reversed_empty_ranges,
        reason = "the test needs an empty range"
    )]
    fn new_rejects_empty_range() {
        let ram = Ram::new(0x2000..=0x1000);

        assert_eq!(ram.err(), Some(EmptyRangeError));
    }

    #[test]
    fn reads_back_written_byte() {
        let mut ram = Ram::new(0x1000..=0x1FFF).unwrap();

        ram.write(0x1234, 0x99);

        assert_eq!(ram.read(0x1234), Some(0x99));
    }

    #[test]
    fn claims_first_and_last_address() {
        let mut ram = Ram::new(0x1000..=0x1FFF).unwrap();

        ram.write(0x1000, 0x11);
        ram.write(0x1FFF, 0x22);

        assert_eq!(ram.read(0x1000), Some(0x11));
        assert_eq!(ram.read(0x1FFF), Some(0x22));
    }

    #[test]
    fn ignores_addresses_outside_range() {
        let mut ram = Ram::new(0x1000..=0x1FFF).unwrap();

        ram.write(0x0FFF, 0x99);
        ram.write(0x2000, 0x99);

        assert_eq!(ram.read(0x0FFF), None);
        assert_eq!(ram.read(0x2000), None);
    }

    #[test]
    fn covers_whole_address_space() {
        let mut ram = Ram::new(0x0000..=0xFFFF).unwrap();

        ram.write(0x0000, 0x11);
        ram.write(0xFFFF, 0x22);

        assert_eq!(ram.read(0x0000), Some(0x11));
        assert_eq!(ram.read(0xFFFF), Some(0x22));
    }
}
