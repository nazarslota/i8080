/// A card that plugs into the [`Backplane`](crate::Backplane).
///
/// A real S-100 card sees every bus cycle. Its address decoder checks the
/// address and either answers or stays silent. `Device` works the same way:
/// the backplane passes every access to every card, and each card returns
/// `None` from a read it doesn't claim and ignores such a write.
///
/// Every method has a default that claims nothing, so a card implements only
/// the address space it uses: a memory card implements `read` and `write`, a
/// serial card `input` and `output`.
pub trait Device {
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
