//! Puts a [`Setup`] into an Altair and starts it, as its owner did with the
//! cards and the front panel.
//!
//! Every Altair here has the 88-2SIO for your terminal and two RAM cards
//! that fill the 64 KB the 8080 addresses. With a PROM, the PROM takes the
//! top page, the last 256 bytes. With disks, the 88-DCDD holds them in
//! drives 0 and up:
//!
//! ```text
//!              image, basic         cpm, disk-basic
//! 0000H-7FFFH  RAM, 32 KB           RAM, 32 KB
//! 8000H-FEFFH  RAM, 32 KB           RAM, 32 KB less a page
//! FF00H-FFFFH                       PROM, 256 bytes
//! ```
//!
//! The I/O ports take no memory: the 8080 reaches them through `IN` and
//! `OUT`, in an address space of their own. `docs/altair/memory-and-ports.md`
//! shows both maps.
//!
//! ```
//! use app::host::console::Console;
//!
//! let image = std::env::temp_dir().join(format!("app-doc-{}-machine.bin", std::process::id()));
//! std::fs::write(&image, [0x76])?; // HLT
//! let setup = app::boot::image::setup(&image)?;
//! let (console, _quit) = Console::start(&setup.tape, setup.erase_key);
//!
//! let altair = app::machine::start(setup, console)?;
//!
//! assert!(altair.is_running());
//! # std::fs::remove_file(&image)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::error::Error;
use std::ops::RangeInclusive;

use altair::Altair;
use altair::cards::{Dcdd, Ram, Rom, Sio2};

use crate::boot::Setup;
use crate::host::console::Console;

/// The first address of the top page, where the PROM card sits.
pub const PROM_ADDRESS: u16 = 0xFF00;
/// The first RAM card: the lower 32 KB.
const LOW_RAM: RangeInclusive<u16> = 0x0000..=0x7FFF;
/// The second RAM card without a PROM: the upper 32 KB.
const HIGH_RAM: RangeInclusive<u16> = 0x8000..=0xFFFF;
/// The second RAM card with a PROM: the upper 32 KB less the PROM page.
const HIGH_RAM_BELOW_PROM: RangeInclusive<u16> = 0x8000..=PROM_ADDRESS - 1;

/// Plugs in the cards for `setup`, wired to `console`, toggles in its
/// program, examines its start address, sets the sense switches and
/// presses RUN. See the module documentation for an example.
pub fn start(setup: Setup, console: Console) -> Result<Altair, Box<dyn Error>> {
    let high_ram = match setup.prom {
        Some(_) => HIGH_RAM_BELOW_PROM,
        None => HIGH_RAM,
    };
    let mut cards = Altair::builder()
        .plug(Ram::new(LOW_RAM)?)
        .plug(Ram::new(high_ram)?)
        .plug(Sio2::new(console)); // The 88-2SIO card at ports 10H and 11H
    if let Some(prom) = setup.prom {
        cards = cards.plug(Rom::new(prom.start, prom.bytes)?);
    }
    if !setup.disks.is_empty() {
        let mut dcdd = Dcdd::new(); // The 88-DCDD at ports 08H to 0AH
        for (drive, disk) in setup.disks.into_iter().enumerate() {
            dcdd.insert(drive, disk)?;
        }
        cards = cards.plug(dcdd);
    }
    let mut altair = cards.build();

    let mut panel = altair.front_panel();
    if let Some(program) = &setup.toggle_in {
        panel.toggle_in(program.start, &program.bytes);
    }
    panel.set_switches(setup.start);
    panel.examine();
    panel.set_switches(u16::from(setup.sense_switches) << 8);
    panel.run();
    Ok(altair)
}
