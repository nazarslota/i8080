//! The bootstrap loader that reads Altair BASIC from a paper tape.
//!
//! A new Altair had nothing in memory. To load BASIC you toggled this
//! 28-byte loader in with the front panel switches, put the BASIC tape in the
//! reader and pressed RUN. The loader reads the tape through the 88-2SIO
//! port A, the same line as the terminal:
//!
//! 1. It skips the leader: bytes equal to the low byte of the loader
//!    address, `C2H` on a BASIC 4.0 tape.
//! 2. It stores the next bytes downward, from the byte below the loader
//!    address to the start of that 256-byte page. They form a second,
//!    larger loader, the checksum loader.
//! 3. It jumps to the checksum loader, which reads BASIC from the rest of
//!    the tape, checks each record and starts BASIC.
//!
//! Each BASIC tape needs its own loader address:
//!
//! | Tape                    | Loader address |
//! |-------------------------|----------------|
//! | 4K BASIC 4.0            | `0FC2H`        |
//! | 8K BASIC 4.0            | `1FC2H`        |
//! | Extended BASIC 4.0, 4.1 | `3FC2H`        |
//!
//! With the tape and the terminal on the 88-2SIO port A, BASIC 4.X expects
//! every sense switch down.
//!
//! The listing, the addresses and the switches come from [Loading Basic with
//! the 88-2SIO Board] by Martin Eberhard (2013). He gives the bytes in octal,
//! as MITS printed them, and so does the code below. His [Altair Software
//! Development using CP/M] describes the tape format.
//!
//! [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf
//! [Altair Software Development using CP/M]: https://deramp.com/downloads/altair/software/papertape_cassette/Checksum%20Loader/Altair%20Paper%20Tape%20Format.pdf

use std::fmt;

use clap::ValueEnum;

/// An Altair BASIC tape that the bootstrap loader reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Basic {
    /// 4K BASIC 4.0.
    #[value(name = "4k-4.0")]
    FourK40,
    /// 8K BASIC 4.0.
    #[value(name = "8k-4.0")]
    EightK40,
    /// Extended BASIC 4.0.
    #[value(name = "extended-4.0")]
    Extended40,
    /// Extended BASIC 4.1.
    #[value(name = "extended-4.1")]
    Extended41,
}

impl Basic {
    /// The sense switches A8 to A15 that BASIC 4.X expects with the 88-2SIO
    /// port A: all down.
    pub const SENSE_SWITCHES: u8 = 0x00;

    /// Returns the loader address: the checksum loader on the tape fills the
    /// bytes below it.
    pub const fn loader_address(self) -> u16 {
        match self {
            Self::FourK40 => 0x0FC2,
            Self::EightK40 => 0x1FC2,
            Self::Extended40 | Self::Extended41 => 0x3FC2,
        }
    }

    /// Returns the byte that the tape's leader repeats: the low byte of the
    /// loader address.
    pub const fn leader(self) -> u8 {
        let [low, _] = self.loader_address().to_le_bytes();
        low
    }
}

impl fmt::Display for Basic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::FourK40 => "4K BASIC 4.0",
            Self::EightK40 => "8K BASIC 4.0",
            Self::Extended40 => "Extended BASIC 4.0",
            Self::Extended41 => "Extended BASIC 4.1",
        };
        f.write_str(name)
    }
}

/// Returns the bootstrap loader that reads the `basic` tape. Load it at
/// 0000H and start the CPU there.
pub fn bootstrap_loader(basic: Basic) -> [u8; 28] {
    let [low, high] = basic.loader_address().to_le_bytes();
    [
        0o076, 0o003, // 000  MVI A,3      reset command for the ACIA
        0o323, 0o020, // 002  OUT 10H      reset the ACIA
        0o076, 0o025, // 004  MVI A,25Q    8 data bits, 1 stop bit
        0o323, 0o020, // 006  OUT 10H      set up the ACIA
        0o041, low, high, // 010  LXI H,loader_address
        0o061, 0o032, 0o000, // 013  LXI SP,032Q  each RET below jumps to 013
        0o333, 0o020, // 016  IN 10H       read the status
        0o017, //        020  RRC          bit 0 (a byte waits) into CY
        0o320, //        021  RNC          no byte: back to 013
        0o333, 0o021, // 022  IN 11H       read the byte
        0o275, //        024  CMP L        a leader byte?
        0o310, //        025  RZ           yes: back to 013
        0o055, //        026  DCR L        next address down
        0o167, //        027  MOV M,A      store the byte
        0o300, //        030  RNZ          more to load: back to 013
        0o351, //        031  PCHL         run the checksum loader
        0o013, 0o000, // 032  DW 013Q      the return address for RET
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_eberhard_listing_for_8k_basic_4_0() {
        let loader = bootstrap_loader(Basic::EightK40);

        assert_eq!(
            loader,
            [
                0o076, 0o003, 0o323, 0o020, 0o076, 0o025, 0o323, 0o020, 0o041, 0o302, 0o037, 0o061,
                0o032, 0o000, 0o333, 0o020, 0o017, 0o320, 0o333, 0o021, 0o275, 0o310, 0o055, 0o167,
                0o300, 0o351, 0o013, 0o000,
            ]
        );
    }

    #[test]
    fn puts_loader_address_into_lxi_h() {
        let loader = bootstrap_loader(Basic::Extended41);

        assert_eq!(loader[8..11], [0o041, 0xC2, 0x3F]); // LXI H,3FC2H
    }

    #[test]
    fn leader_is_low_byte_of_loader_address() {
        let leader = Basic::FourK40.leader();

        assert_eq!(leader, 0xC2);
    }
}
