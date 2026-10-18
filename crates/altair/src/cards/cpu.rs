use core::ops::{Deref, DerefMut};

/// The MITS 88-CPU card: the board that carries the Intel 8080 and its
/// clock.
///
/// The card wraps an [`i8080::Cpu`] and gives you all of its fields and
/// methods. What the card adds to the chip, such as its clock frequency,
/// lives here. Unlike the other cards it doesn't implement [`Card`](super::Card):
/// the CPU drives the bus, and the other cards answer it.
///
/// ```
/// use altair::cards::Cpu;
///
/// let cpu = Cpu::new();
///
/// assert_eq!(Cpu::CLOCK_HZ, 2_000_000);
/// assert_eq!(cpu.registers.pc, 0x0000);
/// assert!(!cpu.is_halted());
/// ```
#[derive(Clone, Debug, Default)]
pub struct Cpu(i8080::Cpu);

impl Cpu {
    /// The clock frequency: the card runs the 8080 at 2 MHz, so it executes
    /// 2 million cycles per second.
    pub const CLOCK_HZ: u64 = 2_000_000;

    /// Creates the card with its 8080 in the power-on state.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Cpu {
    type Target = i8080::Cpu;

    fn deref(&self) -> &i8080::Cpu {
        &self.0
    }
}

impl DerefMut for Cpu {
    fn deref_mut(&mut self) -> &mut i8080::Cpu {
        &mut self.0
    }
}
