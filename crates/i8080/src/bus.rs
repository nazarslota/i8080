//! The buses that connect the CPU to the rest of the machine.
//!
//! The 8080 has two separate address spaces, and each one gets its own trait:
//!
//! - [`MemoryBus`]: 64 KB of memory, reached with 16-bit addresses.
//! - [`IoBus`]: 256 I/O ports, reached only with the `IN` and `OUT`
//!   instructions.
//!
//! On the real chip both spaces share the same address and data pins. At the
//! start of each access the CPU puts a status byte on the bus, and memory and
//! devices read it to learn which space the access targets.

/// The memory address space: 64 KB, one byte per `u16` address.
///
/// You implement this trait for your machine and choose what answers at each
/// address: RAM, ROM or nothing. The methods take `&mut self` because some
/// hardware changes state when the CPU reads it.
pub trait MemoryBus {
    /// Reads the byte at `addr`.
    fn read(&mut self, addr: u16) -> u8;

    /// Writes `value` to `addr`.
    fn write(&mut self, addr: u16, value: u8);
}

/// The I/O address space: 256 ports, used by the `IN` and `OUT` instructions.
///
/// The methods take `&mut self` because reading a port can change the
/// device behind it. Reading a serial card's data port, for example, removes
/// the byte from the card's buffer.
pub trait IoBus {
    /// Reads a byte from `port` (the `IN` instruction).
    fn input(&mut self, port: u8) -> u8;

    /// Writes `value` to `port` (the `OUT` instruction).
    fn output(&mut self, port: u8, value: u8);
}

/// 64 KB of RAM covering the whole address space. Tests and simple machines
/// can use a plain array as their memory.
impl MemoryBus for [u8; 0x10000] {
    fn read(&mut self, addr: u16) -> u8 {
        self[usize::from(addr)]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self[usize::from(addr)] = value;
    }
}

/// An I/O bus with no devices attached.
///
/// Reads return `0xFF`, the value an unconnected port reads on real hardware.
/// `Unconnected` ignores writes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unconnected;

impl IoBus for Unconnected {
    fn input(&mut self, _port: u8) -> u8 {
        0xFF
    }

    fn output(&mut self, _port: u8, _value: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_reads_back_written_byte() {
        let mut memory = [0u8; 0x10000];

        memory.write(0x2000, 0x99);

        assert_eq!(memory.read(0x2000), 0x99);
    }

    #[test]
    fn array_covers_whole_address_space() {
        let mut memory = [0u8; 0x10000];

        memory.write(0x0000, 0x11);
        memory.write(0xFFFF, 0x22);

        assert_eq!(memory.read(0x0000), 0x11);
        assert_eq!(memory.read(0xFFFF), 0x22);
    }

    #[test]
    fn unconnected_port_reads_ff_after_write() {
        let mut io = Unconnected;

        io.output(0x10, 0x00);

        assert_eq!(io.input(0x10), 0xFF);
    }
}
