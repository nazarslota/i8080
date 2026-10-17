/// The 8080 registers that programs can see, apart from the flags.
///
/// The fields are public: any value is valid in any register, so code that
/// drives the CPU (tests, a debugger, the Altair front panel) can read and set
/// them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// The accumulator. Most arithmetic and logic instructions put their
    /// result here.
    pub a: u8,
    /// Register B, the high byte of the `BC` pair.
    pub b: u8,
    /// Register C, the low byte of the `BC` pair.
    pub c: u8,
    /// Register D, the high byte of the `DE` pair.
    pub d: u8,
    /// Register E, the low byte of the `DE` pair.
    pub e: u8,
    /// Register H, the high byte of the `HL` pair.
    pub h: u8,
    /// Register L, the low byte of the `HL` pair.
    pub l: u8,
    /// The stack pointer: the address of the top of the stack.
    pub sp: u16,
    /// The program counter: the address of the next instruction to fetch.
    pub pc: u16,
}
