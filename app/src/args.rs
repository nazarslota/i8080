//! The command line: one subcommand for each way to boot the Altair, and
//! `put` to copy files onto a CP/M disk.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::boot::Basic;

/// The examples at the end of `--help`.
const EXAMPLES: &str = "Examples:
  altair image roms/hello.bin
  altair tape 8k-4.0 \"roms/8K BASIC Ver 4-0.tap\"
  cp roms/cpm22b23-56k.dsk cpm.dsk
  altair disk roms/DBL.HEX cpm.dsk
  altair put cpm.dsk roms/CPUTEST.COM";

/// Runs the emulated Altair 8800 in your terminal. Press Ctrl+] to quit.
#[derive(Parser)]
#[command(name = "altair", version, after_help = EXAMPLES)]
pub struct Args {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,

    /// The sense switches A8 to A15 in hex. Without it, the emulator sets
    /// all of them down, as BASIC 4.X expects with the 88-2SIO.
    ///
    /// Example: --switches 0C
    #[arg(long, global = true, value_parser = parse_switches)]
    pub switches: Option<u8>,

    /// Runs the CPU as fast as your computer can instead of at 2 MHz, for
    /// programs that compute for hours on the real Altair.
    #[arg(long, global = true)]
    pub fast: bool,
}

/// What to do: boot the Altair or put files on a disk.
#[derive(Subcommand)]
pub enum Command {
    /// Boots the Altair.
    #[command(flatten)]
    Boot(BootCommand),

    /// Copies files onto a CP/M disk image, in place of files with the same
    /// names. CP/M then runs a .COM file by its name.
    Put {
        /// The disk image with CP/M 2.2 on it.
        ///
        /// Example: cpm.dsk
        disk: PathBuf,

        /// The files to copy. Each keeps its name, in capitals.
        ///
        /// Example: roms/CPUTEST.COM
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },
}

/// How the Altair boots.
#[derive(Subcommand)]
pub enum BootCommand {
    /// Loads a memory image at 0000H and starts the CPU there.
    Image {
        /// The memory image, up to 64 KB.
        ///
        /// Example: roms/hello.bin
        image: PathBuf,
    },

    /// Loads Altair BASIC from a paper tape. The emulator puts the MITS
    /// bootstrap loader for that BASIC at 0000H, and the loader reads the
    /// tape through the 88-2SIO.
    Tape {
        /// The BASIC on the tape.
        ///
        /// Example: 8k-4.0
        #[arg(value_enum)]
        basic: Basic,

        /// The paper tape.
        ///
        /// Example: "roms/8K BASIC Ver 4-0.tap"
        tape: PathBuf,
    },

    /// Boots from a floppy disk. The emulator puts the disk boot loader
    /// into a PROM at FF00H and the disk into drive 0 of the 88-DCDD, then
    /// starts the CPU in the PROM.
    Disk {
        /// The disk boot loader in Intel HEX, for FF00H to FFFFH.
        ///
        /// Example: roms/DBL.HEX
        loader: PathBuf,

        /// The floppy disk image. The emulator writes to it, so boot a copy.
        ///
        /// Example: cpm.dsk
        disk: PathBuf,
    },
}

/// Parses an 8-bit hex number, with or without the `H` suffix.
fn parse_switches(text: &str) -> Result<u8, String> {
    u8::from_str_radix(text.trim_end_matches(['H', 'h']), 16)
        .map_err(|_| format!("{text} is not a hex byte such as 0C"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_are_consistent() {
        use clap::CommandFactory;

        Args::command().debug_assert();
    }

    #[test]
    fn parse_switches_accepts_h_suffix() {
        assert_eq!(parse_switches("0C"), Ok(0x0C));
        assert_eq!(parse_switches("0CH"), Ok(0x0C));
        assert!(parse_switches("100").is_err());
    }
}
