//! The buses. [`MemoryBus`], [`IoBus`] and [`Bus`] document them.

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

/// A memory bus and an I/O bus joined into one bus for
/// [`Cpu::step`](crate::Cpu::step).
///
/// Memory accesses go to `memory` and port accesses to `io`. Tests and
/// simple machines use it to pair a plain array with [`Unconnected`]:
///
/// ```
/// use i8080::{Bus, Cpu, Unconnected};
///
/// let mut bus = Bus {
///     memory: [0u8; 0x10000],
///     io: Unconnected,
/// };
/// let mut cpu = Cpu::new();
///
/// cpu.step(&mut bus);
///
/// assert_eq!(cpu.registers.pc, 1); // the NOP at 0000H
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct Bus<M, I> {
    /// The memory address space.
    pub memory: M,
    /// The I/O address space.
    pub io: I,
}

impl<M: MemoryBus, I> MemoryBus for Bus<M, I> {
    fn read(&mut self, addr: u16) -> u8 {
        self.memory.read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.memory.write(addr, value);
    }
}

impl<M, I: IoBus> IoBus for Bus<M, I> {
    fn input(&mut self, port: u8) -> u8 {
        self.io.input(port)
    }

    fn output(&mut self, port: u8, value: u8) {
        self.io.output(port, value);
    }
}

/// A borrowed memory bus works as a memory bus, so a [`Bus`] can borrow its
/// memory and leave it with you.
impl<T: MemoryBus + ?Sized> MemoryBus for &mut T {
    fn read(&mut self, addr: u16) -> u8 {
        (**self).read(addr)
    }

    fn write(&mut self, addr: u16, value: u8) {
        (**self).write(addr, value);
    }
}

/// A borrowed I/O bus works as an I/O bus, so a [`Bus`] can borrow its
/// devices and leave them with you.
impl<T: IoBus + ?Sized> IoBus for &mut T {
    fn input(&mut self, port: u8) -> u8 {
        (**self).input(port)
    }

    fn output(&mut self, port: u8, value: u8) {
        (**self).output(port, value);
    }
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
    fn bus_sends_memory_access_to_memory() {
        let mut memory = [0u8; 0x10000];
        let mut bus = Bus {
            memory: &mut memory,
            io: Unconnected,
        };

        bus.write(0x2000, 0x99);

        assert_eq!(bus.read(0x2000), 0x99);
        assert_eq!(memory[0x2000], 0x99);
    }

    #[test]
    fn bus_sends_port_access_to_io() {
        let mut bus = Bus {
            memory: [0u8; 0x10000],
            io: Unconnected,
        };

        bus.output(0x10, 0x00);

        assert_eq!(bus.input(0x10), 0xFF);
    }

    #[test]
    fn unconnected_port_reads_ff_after_write() {
        let mut io = Unconnected;

        io.output(0x10, 0x00);

        assert_eq!(io.input(0x10), 0xFF);
    }
}
