use core::ops::RangeInclusive;

use crate::Device;

/// A memory card: RAM that claims one range of addresses.
///
/// The Altair shipped with 256 bytes of RAM, and owners added memory cards.
/// You pick the range of addresses when you create the card.
///
/// ```
/// use altair::{Backplane, Ram};
/// use i8080::MemoryBus;
///
/// let mut backplane = Backplane::new();
/// backplane.plug(Ram::new(0x0000..=0xFFFF)); // the full 64 KB
///
/// backplane.write(0x2000, 0x99);
///
/// assert_eq!(backplane.read(0x2000), 0x99);
/// ```
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
    /// # Panics
    ///
    /// Panics if `addresses` is empty, such as `0x2000..=0x1000`.
    pub fn new(addresses: RangeInclusive<u16>) -> Self {
        let (start, end) = addresses.into_inner();
        assert!(start <= end, "a memory card needs at least one address");
        let size = usize::from(end - start) + 1;
        Self {
            start,
            bytes: vec![0; size],
        }
    }

    /// Returns the index in `bytes` for `addr`, or `None` if the card doesn't
    /// claim `addr`.
    fn index(&self, addr: u16) -> Option<usize> {
        let index = usize::from(addr.checked_sub(self.start)?);
        (index < self.bytes.len()).then_some(index)
    }
}

impl Device for Ram {
    fn read(&mut self, addr: u16) -> Option<u8> {
        self.index(addr).map(|index| self.bytes[index])
    }

    fn write(&mut self, addr: u16, value: u8) {
        if let Some(index) = self.index(addr) {
            self.bytes[index] = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_fills_memory_with_zeros() {
        let mut ram = Ram::new(0x1000..=0x1FFF);

        let value = ram.read(0x1234);

        assert_eq!(value, Some(0x00));
    }

    #[test]
    #[should_panic(expected = "at least one address")]
    #[allow(
        clippy::reversed_empty_ranges,
        reason = "the test needs an empty range"
    )]
    fn new_rejects_empty_range() {
        Ram::new(0x2000..=0x1000);
    }

    #[test]
    fn reads_back_written_byte() {
        let mut ram = Ram::new(0x1000..=0x1FFF);

        ram.write(0x1234, 0x99);

        assert_eq!(ram.read(0x1234), Some(0x99));
    }

    #[test]
    fn claims_first_and_last_address() {
        let mut ram = Ram::new(0x1000..=0x1FFF);

        ram.write(0x1000, 0x11);
        ram.write(0x1FFF, 0x22);

        assert_eq!(ram.read(0x1000), Some(0x11));
        assert_eq!(ram.read(0x1FFF), Some(0x22));
    }

    #[test]
    fn ignores_addresses_outside_range() {
        let mut ram = Ram::new(0x1000..=0x1FFF);

        ram.write(0x0FFF, 0x99);
        ram.write(0x2000, 0x99);

        assert_eq!(ram.read(0x0FFF), None);
        assert_eq!(ram.read(0x2000), None);
    }

    #[test]
    fn covers_whole_address_space() {
        let mut ram = Ram::new(0x0000..=0xFFFF);

        ram.write(0x0000, 0x11);
        ram.write(0xFFFF, 0x22);

        assert_eq!(ram.read(0x0000), Some(0x11));
        assert_eq!(ram.read(0xFFFF), Some(0x22));
    }
}
