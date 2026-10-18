//! The command line: a subcommand for each way to boot the Altair, and
//! `disk` for the CP/M disk tools.
//!
//! ```
//! use app::cli::{Args, Command};
//! use clap::Parser;
//!
//! let args = Args::parse_from(["altair", "--fast", "cpm", "a.dsk", "b.dsk"]);
//!
//! assert!(args.fast);
//! let Command::Cpm { prom, disks } = args.command else { panic!() };
//! assert_eq!(prom.to_str(), Some("roms/DBL.HEX"));
//! assert_eq!(disks.len(), 2);
//! ```

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::boot::basic::Basic;

/// The drives that the CP/M BIOS knows, A: to D:.
const CPM_DRIVES: usize = 4;

/// The examples at the end of `--help`.
const EXAMPLES: &str = "Examples:
  altair image roms/hello.bin
  altair basic 8k-4.0 \"roms/8K BASIC Ver 4-0.tap\"
  cp roms/cpm22b23-56k.dsk a.dsk
  altair cpm a.dsk
  altair disk format b.dsk
  altair disk put b.dsk roms/CPUTEST.COM
  altair disk ls b.dsk
  altair cpm a.dsk b.dsk";

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

/// What to do: boot the Altair one of three ways, or work on a CP/M disk.
#[derive(Subcommand)]
pub enum Command {
    /// Loads a memory image at 0000H and starts the CPU there.
    Image {
        /// The memory image, up to 64 KB.
        ///
        /// Example: roms/hello.bin
        image: PathBuf,
    },

    /// Loads Altair BASIC from a paper tape. The emulator toggles in the
    /// MITS bootstrap loader for that BASIC at 0000H, and the loader reads
    /// the tape through the 88-2SIO.
    Basic {
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

    /// Boots CP/M from floppy disks. The emulator puts the disk boot loader
    /// into a PROM at FF00H and the disks into the drives of the 88-DCDD,
    /// then starts the CPU in the PROM.
    Cpm {
        /// The disk boot loader in Intel HEX, for FF00H to FFFFH.
        #[arg(long, default_value = "roms/DBL.HEX")]
        prom: PathBuf,

        /// The floppy disk images for drives A: to D:, the boot disk first.
        /// The emulator writes to them, so boot copies.
        ///
        /// Example: a.dsk b.dsk
        #[arg(required = true, num_args = 1..=CPM_DRIVES)]
        disks: Vec<PathBuf>,
    },

    /// Makes, fills and lists CP/M disk images. Doesn't start the Altair.
    #[command(subcommand)]
    Disk(DiskCommand),
}

/// What to do with a CP/M disk image.
#[derive(Subcommand)]
pub enum DiskCommand {
    /// Makes an empty disk image, with no system on it, for drives B: to
    /// D:.
    Format {
        /// The new disk image. The file must not exist yet.
        ///
        /// Example: b.dsk
        disk: PathBuf,
    },

    /// Copies files onto a disk image, in place of files with the same
    /// names. CP/M then runs a .COM file by its name.
    Put {
        /// The disk image.
        ///
        /// Example: b.dsk
        disk: PathBuf,

        /// The files to copy. Each keeps its name, in capitals.
        ///
        /// Example: roms/CPUTEST.COM
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Lists the files on a disk image and the free space.
    Ls {
        /// The disk image.
        ///
        /// Example: b.dsk
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
