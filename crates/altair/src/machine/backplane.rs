//! The S-100 backplane. [`Backplane`] documents it.

use i8080::{IoBus, MemoryBus};

use crate::cards::Card;

/// What a read returns when no card claims the address or port. No card
/// drives the data lines, so the CPU reads all 1s.
const OPEN_BUS: u8 = 0xFF;

/// The S-100 backplane: a row of slots that connects the CPU to the cards.
///
/// `Backplane` implements [`MemoryBus`] and [`IoBus`], so you lend it to
/// [`Cpu::step`](i8080::Cpu::step) as the CPU's bus. It passes each access to
/// the cards in the order you plugged them in. On a read, the first card that
/// claims the address answers. On a write, every card that claims the address
/// receives the byte.
#[derive(Default)]
pub struct Backplane {
    cards: Vec<Box<dyn Card>>,
}

impl Backplane {
    /// Creates a backplane with empty slots.
    ///
    /// ```
    /// use altair::Backplane;
    /// use i8080::MemoryBus;
    ///
    /// let mut backplane = Backplane::new();
    ///
    /// assert_eq!(backplane.read(0x1234), 0xFF); // an empty slot
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Plugs `card` into the next free slot.
    ///
    /// ```
    /// use altair::Backplane;
    /// use altair::cards::Ram;
    /// use i8080::MemoryBus;
    ///
    /// let mut backplane = Backplane::new();
    /// backplane.plug(Ram::new(0x0000..=0x00FF)?);
    ///
    /// backplane.write(0x0010, 0x99);
    ///
    /// assert_eq!(backplane.read(0x0010), 0x99);
    /// assert_eq!(backplane.read(0x0100), 0xFF); // past the card
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn plug(&mut self, card: impl Card + 'static) {
        self.cards.push(Box::new(card));
    }

    /// Samples every card and returns whether any card requests an interrupt.
    ///
    /// The PINT line combines the requests. Sampling neither picks a winner
    /// nor clears a request: the handler must service the cards.
    ///
    /// ```
    /// use altair::{Backplane, cards::Card};
    ///
    /// struct Request;
    /// impl Card for Request {
    ///     fn interrupt_pending(&mut self) -> bool { true }
    /// }
    ///
    /// let mut backplane = Backplane::new();
    /// backplane.plug(Request);
    ///
    /// assert!(backplane.interrupt_pending());
    /// ```
    pub fn interrupt_pending(&mut self) -> bool {
        let mut pending = false;
        for card in &mut self.cards {
            pending |= card.interrupt_pending();
        }
        pending
    }
}

impl MemoryBus for Backplane {
    fn read(&mut self, addr: u16) -> u8 {
        self.cards
            .iter_mut()
            .find_map(|card| card.read(addr))
            .unwrap_or(OPEN_BUS)
    }

    fn write(&mut self, addr: u16, value: u8) {
        for card in &mut self.cards {
            card.write(addr, value);
        }
    }
}

impl IoBus for Backplane {
    fn input(&mut self, port: u8) -> u8 {
        self.cards
            .iter_mut()
            .find_map(|card| card.input(port))
            .unwrap_or(OPEN_BUS)
    }

    fn output(&mut self, port: u8, value: u8) {
        for card in &mut self.cards {
            card.output(port, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A card that claims one memory address and one I/O port and answers
    /// with the last byte written there.
    struct Latch {
        addr: u16,
        port: u8,
        value: u8,
    }

    impl Latch {
        fn new(addr: u16, port: u8, value: u8) -> Self {
            Self { addr, port, value }
        }
    }

    impl Card for Latch {
        fn read(&mut self, addr: u16) -> Option<u8> {
            (addr == self.addr).then_some(self.value)
        }

        fn write(&mut self, addr: u16, value: u8) {
            if addr == self.addr {
                self.value = value;
            }
        }

        fn input(&mut self, port: u8) -> Option<u8> {
            (port == self.port).then_some(self.value)
        }

        fn output(&mut self, port: u8, value: u8) {
            if port == self.port {
                self.value = value;
            }
        }
    }

    #[test]
    fn empty_backplane_reads_open_bus() {
        let mut backplane = Backplane::new();

        let memory = backplane.read(0x1234);
        let port = backplane.input(0x12);

        assert_eq!(memory, 0xFF);
        assert_eq!(port, 0xFF);
    }

    #[test]
    fn cards_without_interrupts_leave_pint_inactive() {
        let mut backplane = Backplane::new();
        let empty = backplane.interrupt_pending();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));

        let pending = backplane.interrupt_pending();

        assert!(!empty);
        assert!(!pending);
    }

    #[test]
    fn interrupt_samples_every_card_and_keeps_request_active() {
        use std::cell::Cell;
        use std::rc::Rc;

        struct Request {
            active: bool,
            samples: Rc<Cell<usize>>,
        }

        impl Card for Request {
            fn interrupt_pending(&mut self) -> bool {
                self.samples.set(self.samples.get() + 1);
                self.active
            }
        }

        let samples = Rc::new(Cell::new(0));
        let mut backplane = Backplane::new();
        for active in [true, false, true] {
            backplane.plug(Request {
                active,
                samples: Rc::clone(&samples),
            });
        }

        let first = backplane.interrupt_pending();
        let second = backplane.interrupt_pending();

        assert!(first);
        assert!(second);
        assert_eq!(samples.get(), 6);
    }

    #[test]
    fn read_reaches_card_that_claims_address() {
        let mut backplane = Backplane::new();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));
        backplane.plug(Latch::new(0x2000, 0x20, 0x22));

        let value = backplane.read(0x2000);

        assert_eq!(value, 0x22);
    }

    #[test]
    fn write_reaches_card_that_claims_address() {
        let mut backplane = Backplane::new();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));

        backplane.write(0x1000, 0x99);

        assert_eq!(backplane.read(0x1000), 0x99);
    }

    #[test]
    fn input_reaches_card_that_claims_port() {
        let mut backplane = Backplane::new();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));
        backplane.plug(Latch::new(0x2000, 0x20, 0x22));

        let value = backplane.input(0x20);

        assert_eq!(value, 0x22);
    }

    #[test]
    fn output_reaches_card_that_claims_port() {
        let mut backplane = Backplane::new();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));

        backplane.output(0x10, 0x99);

        assert_eq!(backplane.input(0x10), 0x99);
    }

    #[test]
    fn first_plugged_card_answers_shared_address() {
        let mut backplane = Backplane::new();
        backplane.plug(Latch::new(0x1000, 0x10, 0x11));
        backplane.plug(Latch::new(0x1000, 0x10, 0x22));

        let value = backplane.read(0x1000);

        assert_eq!(value, 0x11);
    }
}
