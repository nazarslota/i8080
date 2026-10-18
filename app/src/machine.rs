//! The Altair that `app` builds for each way to boot, and what its owner
//! did on the front panel to start it.
//!
//! | Boot  | Cards beyond the CPU, memory and 88-2SIO | Starts at |
//! |-------|------------------------------------------|-----------|
//! | image | none                                     | 0000H     |
//! | tape  | none                                     | 0000H     |
//! | disk  | the PROM at FF00H and the 88-DCDD        | FF00H     |
//!
//! Two RAM cards fill the 64 KB that the 8080 addresses. With a disk, the
//! PROM takes the top page, the last 256 bytes:
//!
//! ```text
//!              image, tape          disk
//! 0000H-7FFFH  RAM, 32 KB           RAM, 32 KB
//! 8000H-FEFFH  RAM, 32 KB           RAM, 32 KB less a page
//! FF00H-FFFFH                       PROM, 256 bytes
//! ```
//!
//! The I/O ports take no memory: the 8080 reaches them through `IN` and
//! `OUT`, in an address space of their own. `docs/altair/memory-and-ports.md`
//! shows both maps.

use std::error::Error;
use std::ops::RangeInclusive;

use altair::cards::{Cpu, Dcdd, EmptyRangeError, NoSuchDriveError, Ram, Rom, Sio2};
use altair::{Altair, AltairBuilder};

use crate::boot::{Boot, bootstrap_loader};
use crate::console::Console;

/// Where you toggle in an image or the bootstrap loader, and where the CPU
/// starts it: the first address of memory.
const PROGRAM_ADDRESS: u16 = 0x0000;
/// The first address of the top page, where the PROM card sits.
pub const PROM_ADDRESS: u16 = 0xFF00;
/// The first RAM card: the lower 32 KB.
const LOW_RAM: RangeInclusive<u16> = 0x0000..=0x7FFF;
/// The second RAM card without a PROM: the upper 32 KB.
const HIGH_RAM: RangeInclusive<u16> = 0x8000..=0xFFFF;
/// The second RAM card with a PROM: the upper 32 KB less the PROM page.
const HIGH_RAM_BELOW_PROM: RangeInclusive<u16> = 0x8000..=PROM_ADDRESS - 1;
/// The 88-DCDD drive that holds the disk, the one the boot loader reads.
const BOOT_DRIVE: usize = 0;

/// Builds the Altair for `boot`, wired to `console`, and starts it with the
/// sense switches at `sense_switches`.
pub fn start(boot: Boot, console: Console, sense_switches: u8) -> Result<Altair, Box<dyn Error>> {
    let altair = match boot {
        Boot::Image(image) => {
            let altair = cards(console, HIGH_RAM)?.build()?;
            toggle_in_and_run(altair, &image, sense_switches)
        }
        Boot::Tape { basic, .. } => {
            let altair = cards(console, HIGH_RAM)?.build()?;
            toggle_in_and_run(altair, &bootstrap_loader(basic), sense_switches)
        }
        Boot::Disk { loader, disk } => {
            let start = loader.start;
            let altair = cards(console, HIGH_RAM_BELOW_PROM)?
                .plug(Rom::new(loader.start, loader.bytes)?) // The PROM card
                .plug(disk_controller(disk)?) // The 88-DCDD at ports 08H to 0AH
                .build()?;
            run_from(altair, start, sense_switches)
        }
    };
    Ok(altair)
}

/// Plugs in the cards every Altair here has: the CPU, a RAM card for
/// [`LOW_RAM`], a RAM card for `high_ram` and the serial card for `console`.
fn cards(
    console: Console,
    high_ram: RangeInclusive<u16>,
) -> Result<AltairBuilder, EmptyRangeError> {
    Ok(Altair::builder()
        .plug(Cpu::new()) // The 88-CPU card
        .plug(Ram::new(LOW_RAM)?)
        .plug(Ram::new(high_ram)?)
        .plug(Sio2::new(console))) // The 88-2SIO card at ports 10H and 11H
}

/// Returns an 88-DCDD with `disk` in the boot drive.
fn disk_controller(disk: Vec<u8>) -> Result<Dcdd, NoSuchDriveError> {
    let mut dcdd = Dcdd::new();
    dcdd.insert(BOOT_DRIVE, disk)?;
    Ok(dcdd)
}

/// Toggles `program` in at [`PROGRAM_ADDRESS`] with the front panel and
/// runs it from there.
fn toggle_in_and_run(mut altair: Altair, program: &[u8], sense_switches: u8) -> Altair {
    altair.front_panel().toggle_in(PROGRAM_ADDRESS, program);
    run_from(altair, PROGRAM_ADDRESS, sense_switches)
}

/// Examines `start`, sets the sense switches and presses RUN.
fn run_from(mut altair: Altair, start: u16, sense_switches: u8) -> Altair {
    let mut panel = altair.front_panel();
    panel.set_switches(start);
    panel.examine();
    panel.set_switches(u16::from(sense_switches) << 8);
    panel.run();
    altair
}
