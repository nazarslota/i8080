//! The command line.

use std::path::PathBuf;

use clap::{ArgGroup, Parser};

use crate::loader::Basic;

/// The examples at the end of `--help`.
const EXAMPLES: &str = "Examples:
  altair roms/hello.bin
  altair --tape \"roms/8K BASIC Ver 4-0.tap\" --basic 8k-4.0";

/// Runs the emulated Altair 8800 in your terminal. Press Ctrl+] to quit.
#[derive(Parser)]
#[command(name = "altair", version, after_help = EXAMPLES)]
#[command(group(ArgGroup::new("program").required(true).args(["image", "tape"])))]
pub struct Args {
    /// A memory image to load at 0000H. The CPU starts there.
    ///
    /// Example: roms/hello.bin
    pub image: Option<PathBuf>,

    /// A paper tape with Altair BASIC for the reader. The emulator puts the
    /// MITS bootstrap loader for that BASIC at 0000H, and the loader reads
    /// the tape through the 88-2SIO.
    ///
    /// Example: --tape "roms/8K BASIC Ver 4-0.tap"
    #[arg(long, requires = "basic")]
    pub tape: Option<PathBuf>,

    /// The BASIC on the tape.
    ///
    /// Example: --basic 8k-4.0
    #[arg(long, value_enum, requires = "tape")]
    pub basic: Option<Basic>,

    /// The sense switches A8 to A15 in hex. Without it, the emulator sets
    /// all of them down, as BASIC 4.X expects with the 88-2SIO.
    ///
    /// Example: --switches 0C
    #[arg(long, value_parser = parse_switches)]
    pub switches: Option<u8>,
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
