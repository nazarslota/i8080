/// The other end of the serial cable: the terminal you type on.
///
/// A serial card sends what the program prints to the terminal and takes
/// the keys you type from it. The `altair` crate does no host I/O, so the
/// application implements this trait and connects it to your real keyboard
/// and screen.
pub trait Terminal {
    /// Returns the next key you typed, or `None` if you typed nothing.
    fn receive(&mut self) -> Option<u8>;

    /// Shows `byte` on the screen.
    fn send(&mut self, byte: u8);
}
