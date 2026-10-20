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
use crate::boot::{cpm, disk_basic, dos};

/// The examples at the end of `--help`.
const EXAMPLES: &str = r#"Examples:
  Memory image:
    altair image roms/hello.bin

  BASIC from tape:
    altair basic 4k-4.0 "roms/4K BASIC Ver 4-0.tap"
    altair basic 8k-4.0 "roms/8K BASIC Ver 4-0.tap"
    altair basic extended-4.1 "roms/Extended BASIC Ver 4-1.tap"

  CP/M:
    cp roms/cpm22b23-56k.dsk a.dsk
    altair cpm a.dsk

  Disk BASIC:
    cp "roms/Disk BASIC 300-5-F.dsk" basic.dsk
    altair disk-basic basic.dsk

  Altair DOS:
    cp roms/AltairDOS.dsk dos.dsk
    altair dos dos.dsk

  Add a CP/M data disk:
    altair disk format b.dsk
    altair disk put b.dsk roms/TST8080.COM roms/CPUTEST.COM
    altair disk ls b.dsk
    altair cpm a.dsk b.dsk

  Run without the 2 MHz limit:
    altair --fast cpm a.dsk b.dsk
    altair --fast disk-basic basic.dsk
    altair --fast dos dos.dsk

  Set the boot PROM and sense switches:
    altair --switches 10 disk-basic --prom roms/DBL.HEX basic.dsk
"#;

/// Runs the emulated Altair 8800 in your terminal. Press Ctrl+] to quit.
#[derive(Parser)]
#[command(name = "altair", version, after_help = EXAMPLES)]
pub struct Args {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,

    /// Overrides the sense switches A8 to A15 in hex. The boot command sets
    /// the default: 10H for Disk BASIC and DOS, 00H for the other commands.
    ///
    /// Example: --switches 0C
    #[arg(long, global = true, value_parser = parse_switches)]
    pub switches: Option<u8>,

    /// Runs the CPU as fast as your computer can instead of at 2 MHz, for
    /// programs that compute for hours on the real Altair.
    #[arg(long, global = true)]
    pub fast: bool,
}

/// What to do: boot the Altair, or work on a CP/M disk.
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
        #[arg(required = true, num_args = 1..=cpm::DRIVE_COUNT)]
        disks: Vec<PathBuf>,
    },

    /// Boots Altair Disk BASIC from floppy disks, with sense switches 10H
    /// for the 88-2SIO terminal. Uses the DBL PROM at FF00H.
    DiskBasic {
        /// The disk boot loader in Intel HEX, for FF00H to FFFFH.
        #[arg(long, default_value = "roms/DBL.HEX")]
        prom: PathBuf,

        /// The images for drives 0 to 15, the Disk BASIC boot disk first.
        /// The emulator writes to them, so boot copies.
        ///
        /// Example: basic.dsk
        #[arg(required = true, num_args = 1..=disk_basic::DRIVE_COUNT)]
        disks: Vec<PathBuf>,
    },

    /// Boots Altair DOS from floppy disks, with sense switches 10H for the
    /// 88-2SIO terminal. Uses the DBL PROM at FF00H.
    Dos {
        /// The disk boot loader in Intel HEX, for FF00H to FFFFH.
        #[arg(long, default_value = "roms/DBL.HEX")]
        prom: PathBuf,

        /// The images for drives 0 to 5, the DOS boot disk first.
        /// The emulator writes to them, so boot copies.
        ///
        /// Example: dos.dsk
        #[arg(required = true, num_args = 1..=dos::DRIVE_COUNT)]
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
    fn disk_basic_uses_default_prom_and_keeps_disk_order() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["altair", "disk-basic", "basic.dsk", "data.dsk"])?;

        let Command::DiskBasic { prom, disks } = args.command else {
            panic!("expected disk-basic");
        };

        assert_eq!(prom, PathBuf::from("roms/DBL.HEX"));
        assert_eq!(
            disks,
            [PathBuf::from("basic.dsk"), PathBuf::from("data.dsk")]
        );
        assert!(args.switches.is_none());
        Ok(())
    }

    #[test]
    fn disk_basic_accepts_custom_prom_and_switch_override() -> Result<(), clap::Error> {
        let args = Args::try_parse_from([
            "altair",
            "--switches",
            "00",
            "disk-basic",
            "--prom",
            "loader.hex",
            "basic.dsk",
        ])?;

        let Command::DiskBasic { prom, disks } = args.command else {
            panic!("expected disk-basic");
        };

        assert_eq!(prom, PathBuf::from("loader.hex"));
        assert_eq!(disks, [PathBuf::from("basic.dsk")]);
        assert_eq!(args.switches, Some(0x00));
        Ok(())
    }

    #[test]
    fn dos_uses_default_prom_and_keeps_disk_order() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["altair", "dos", "dos.dsk", "data.dsk"])?;

        let Command::Dos { prom, disks } = args.command else {
            panic!("expected dos");
        };

        assert_eq!(prom, PathBuf::from("roms/DBL.HEX"));
        assert_eq!(disks, [PathBuf::from("dos.dsk"), PathBuf::from("data.dsk")]);
        assert!(args.switches.is_none());
        Ok(())
    }

    #[test]
    fn dos_accepts_custom_prom_and_switch_override() -> Result<(), clap::Error> {
        let args = Args::try_parse_from([
            "altair",
            "--switches",
            "00",
            "dos",
            "--prom",
            "loader.hex",
            "dos.dsk",
        ])?;

        let Command::Dos { prom, disks } = args.command else {
            panic!("expected dos");
        };

        assert_eq!(prom, PathBuf::from("loader.hex"));
        assert_eq!(disks, [PathBuf::from("dos.dsk")]);
        assert_eq!(args.switches, Some(0x00));
        Ok(())
    }

    #[test]
    fn disk_boot_commands_enforce_their_drive_counts() {
        for (command, maximum) in [("cpm", 4), ("disk-basic", 16), ("dos", 6)] {
            let empty = Args::try_parse_from(["altair", command]);
            let mut args = vec!["altair", command];
            args.extend(std::iter::repeat_n("disk.dsk", maximum));

            let full = Args::try_parse_from(&args);
            args.push("extra.dsk");
            let excess = Args::try_parse_from(&args);

            assert!(empty.is_err(), "command {command}");
            assert!(full.is_ok(), "command {command}");
            assert!(excess.is_err(), "command {command}");
        }
    }

    #[test]
    fn parse_switches_accepts_h_suffix() {
        assert_eq!(parse_switches("0C"), Ok(0x0C));
        assert_eq!(parse_switches("0CH"), Ok(0x0C));
        assert!(parse_switches("100").is_err());
    }
}
