//! The MITS 88-DCDD floppy disk controller: up to 16 eight-inch drives.
//!
//! A program talks to the controller through three I/O ports:
//!
//! | Port  | `IN`                    | `OUT`                           |
//! |-------|-------------------------|---------------------------------|
//! | `08H` | the drive status        | selects a drive, or 80H for none |
//! | `09H` | the sector under the head | a command: step, load the head, write |
//! | `0AH` | the next byte read      | the next byte to write          |
//!
//! The status bits are active low: a 0 bit means yes.
//!
//! | Bit | Mask  | 0 when                                  |
//! |-----|-------|-----------------------------------------|
//! | 0   | `01H` | the controller takes the next write byte |
//! | 1   | `02H` | the head may move                       |
//! | 2   | `04H` | the head is loaded                      |
//! | 3   | `08H` | a drive is selected and holds a disk    |
//! | 6   | `40H` | the head sits on track 0                |
//! | 7   | `80H` | a byte waits in port `0AH`              |
//!
//! A disk has 77 tracks of 32 sectors, and each sector holds 137 bytes: a
//! 128-byte CP/M record plus the track and sector numbers, a checksum and a
//! stop byte. A program finds a sector by reading port `09H` until the sector
//! it wants passes under the head, then reads or writes its 137 bytes.
//!
//! The real disk spins on its own, and a program has 32 microseconds per
//! byte. The emulated disk moves on each read of port `09H` instead, so the
//! next sector comes up every time a program asks, and a byte always waits.
//!
//! The ports and bits come from the BIOS of [Altair CP/M 2.2b] by Mike
//! Douglas, and from the [Altair Disk Boot Loader] that Martin Eberhard
//! disassembled.
//!
//! [Altair CP/M 2.2b]: https://deramp.com/downloads/altair/software/8_inch_floppy/CPM/CPM%202.2/CPM%202.2B/BIOS.ASM
//! [Altair Disk Boot Loader]: https://deramp.com/downloads/altair/software/roms/orginal_roms/DBL.ASM

use super::Card;

/// The port that selects a drive on `OUT` and returns the status on `IN`.
const SELECT_PORT: u8 = 0x08;
/// The port that takes a command on `OUT` and returns the sector position on
/// `IN`.
const COMMAND_PORT: u8 = 0x09;
/// The port that returns the next byte read on `IN` and takes the next byte
/// to write on `OUT`.
const DATA_PORT: u8 = 0x0A;

/// The select value that deselects every drive.
const DESELECT: u8 = 0x80;
/// The bits of the select value that pick one of the 16 drives.
const DRIVE_MASK: u8 = 0x0F;

/// Command bit: step the head one track in, toward track 76.
const STEP_IN_COMMAND: u8 = 0x01;
/// Command bit: step the head one track out, toward track 0.
const STEP_OUT_COMMAND: u8 = 0x02;
/// Command bit: load the head onto the disk.
const HEAD_LOAD_COMMAND: u8 = 0x04;
/// Command bit: lift the head off the disk.
const HEAD_UNLOAD_COMMAND: u8 = 0x08;
/// Command bit: start writing the sector under the head.
const WRITE_ENABLE_COMMAND: u8 = 0x80;

/// Status bit 0: 0 when the controller takes the next write byte.
const WRITE_READY_BIT: u8 = 0b0000_0001;
/// Status bit 1: 0 when the head may move.
const MOVE_OK_BIT: u8 = 0b0000_0010;
/// Status bit 2: 0 when the head is loaded.
const HEAD_LOADED_BIT: u8 = 0b0000_0100;
/// Status bit 3: 0 when a drive is selected and holds a disk.
const ENABLED_BIT: u8 = 0b0000_1000;
/// Status bit 6: 0 when the head sits on track 0.
const TRACK_0_BIT: u8 = 0b0100_0000;
/// Status bit 7: 0 when a byte waits in the data port.
const READ_READY_BIT: u8 = 0b1000_0000;
/// Every status bit at 1: no drive answers.
const NO_DRIVE_STATUS: u8 = 0xFF;

/// The sector position bits 6 and 7, which a program ignores.
const SECTOR_UNUSED_BITS: u8 = 0b1100_0000;

/// The number of drives the controller can select.
const DRIVES: usize = 16;
/// The tracks on a disk, 0 to 76.
const TRACKS: usize = 77;
/// The sectors on a track, 0 to 31.
const SECTORS: usize = 32;
/// The bytes in a sector.
const SECTOR_BYTES: usize = 137;
/// The bytes on a disk: 77 tracks of 32 sectors of 137 bytes.
pub const DISK_BYTES: usize = TRACKS * SECTORS * SECTOR_BYTES;

/// What a read returns past the end of a short disk image.
const BLANK_BYTE: u8 = 0x00;

/// The MITS 88-DCDD floppy disk controller with its drives.
///
/// You insert a disk image into a drive, and a program reads and writes it
/// through ports `08H` to `0AH`. The controller keeps the disk in memory:
/// [`disk`](Dcdd::disk) returns it with every write the program made.
///
/// ```
/// use altair::cards::{Card, Dcdd};
///
/// let mut disk = vec![0; altair::cards::DISK_BYTES];
/// disk[0] = 0x42; // track 0, sector 0, byte 0
/// let mut dcdd = Dcdd::new();
/// dcdd.insert(0, disk);
///
/// dcdd.output(0x08, 0x00); // select drive 0
/// dcdd.output(0x09, 0x04); // load the head
/// let sector = dcdd.input(0x09); // sector 0 passes under the head
///
/// assert_eq!(sector, Some(0b1100_0000)); // sector 0, bit 0 at 0: sector true
/// assert_eq!(dcdd.input(0x0A), Some(0x42));
/// ```
pub struct Dcdd {
    drives: Vec<Drive>,
    selected: Option<usize>,
}

/// One drive: the disk in it and where its head is.
struct Drive {
    disk: Option<Vec<u8>>,
    track: usize,
    head_loaded: bool,
    /// The sector under the head. A new drive starts at the last sector, so
    /// the first read of the sector position brings sector 0.
    sector: usize,
    /// The next byte to read or write within the sector.
    byte: usize,
    writing: bool,
}

impl Dcdd {
    /// Creates a controller with 16 empty drives.
    pub fn new() -> Self {
        Self {
            drives: (0..DRIVES).map(|_| Drive::new()).collect(),
            selected: None,
        }
    }

    /// Inserts `disk` into drive `drive`, 0 to 15.
    ///
    /// # Panics
    ///
    /// Panics if `drive` is 16 or more.
    pub fn insert(&mut self, drive: usize, disk: Vec<u8>) {
        self.drives[drive].disk = Some(disk);
    }

    /// Returns the disk in drive `drive`, with every write the program made,
    /// or `None` for an empty drive.
    pub fn disk(&self, drive: usize) -> Option<&[u8]> {
        self.drives.get(drive)?.disk.as_deref()
    }

    /// Returns the selected drive if it holds a disk.
    fn drive(&mut self) -> Option<&mut Drive> {
        let drive = &mut self.drives[self.selected?];
        drive.disk.as_ref()?;
        Some(drive)
    }

    /// Selects a drive, or deselects them all for 80H.
    fn select(&mut self, value: u8) {
        self.selected = if value & DESELECT == 0 {
            Some(usize::from(value & DRIVE_MASK))
        } else {
            None
        };
    }

    /// Returns the status byte, every bit active low.
    fn status(&mut self) -> u8 {
        let Some(drive) = self.drive() else {
            return NO_DRIVE_STATUS;
        };

        let mut status = NO_DRIVE_STATUS & !(ENABLED_BIT | MOVE_OK_BIT);
        if drive.head_loaded {
            status &= !(HEAD_LOADED_BIT | READ_READY_BIT);
        }
        if drive.writing {
            status &= !WRITE_READY_BIT;
        }
        if drive.track == 0 {
            status &= !TRACK_0_BIT;
        }
        status
    }

    /// Carries out the command bits in `value`.
    fn command(&mut self, value: u8) {
        let Some(drive) = self.drive() else { return };

        if value & STEP_IN_COMMAND != 0 && drive.track < TRACKS - 1 {
            drive.track += 1;
        }
        if value & STEP_OUT_COMMAND != 0 && drive.track > 0 {
            drive.track -= 1;
        }
        if value & HEAD_LOAD_COMMAND != 0 {
            drive.head_loaded = true;
        }
        if value & HEAD_UNLOAD_COMMAND != 0 {
            drive.head_loaded = false;
        }
        if value & WRITE_ENABLE_COMMAND != 0 {
            drive.writing = true;
            drive.byte = 0;
        }
    }

    /// Moves the next sector under the head and returns its number in bits
    /// 1 to 5, with bit 0 at 0: the sector is true.
    fn sector_position(&mut self) -> u8 {
        let Some(drive) = self.drive() else {
            return NO_DRIVE_STATUS;
        };
        if !drive.head_loaded {
            return NO_DRIVE_STATUS;
        }

        drive.sector = (drive.sector + 1) % SECTORS;
        drive.byte = 0;
        drive.writing = false;
        let sector = u8::try_from(drive.sector).expect("a sector number fits in 5 bits");
        SECTOR_UNUSED_BITS | sector << 1
    }

    /// Returns the next byte of the sector under the head.
    fn read(&mut self) -> u8 {
        let Some(drive) = self.drive() else {
            return NO_DRIVE_STATUS;
        };
        let Some(offset) = drive.offset() else {
            return BLANK_BYTE;
        };

        drive.byte += 1;
        let disk = drive.disk.as_ref().expect("drive() checked the disk");
        disk.get(offset).copied().unwrap_or(BLANK_BYTE)
    }

    /// Writes `value` as the next byte of the sector under the head.
    fn write(&mut self, value: u8) {
        let Some(drive) = self.drive() else { return };
        if !drive.writing {
            return;
        }
        let Some(offset) = drive.offset() else {
            drive.writing = false;
            return;
        };

        drive.byte += 1;
        let disk = drive.disk.as_mut().expect("drive() checked the disk");
        if offset >= disk.len() {
            disk.resize(offset + 1, BLANK_BYTE);
        }
        disk[offset] = value;
    }
}

impl Default for Dcdd {
    fn default() -> Self {
        Self::new()
    }
}

impl Drive {
    /// Creates an empty drive with its head on track 0.
    fn new() -> Self {
        Self {
            disk: None,
            track: 0,
            head_loaded: false,
            sector: SECTORS - 1,
            byte: 0,
            writing: false,
        }
    }

    /// Returns the position of the next byte in the disk image, or `None`
    /// past the end of the sector.
    fn offset(&self) -> Option<usize> {
        (self.byte < SECTOR_BYTES)
            .then(|| (self.track * SECTORS + self.sector) * SECTOR_BYTES + self.byte)
    }
}

impl Card for Dcdd {
    fn input(&mut self, port: u8) -> Option<u8> {
        match port {
            SELECT_PORT => Some(self.status()),
            COMMAND_PORT => Some(self.sector_position()),
            DATA_PORT => Some(self.read()),
            _ => None,
        }
    }

    fn output(&mut self, port: u8, value: u8) {
        match port {
            SELECT_PORT => self.select(value),
            COMMAND_PORT => self.command(value),
            DATA_PORT => self.write(value),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A disk whose every byte holds the low byte of its position, so a test
    /// can tell which track, sector and byte it read.
    fn numbered_disk() -> Vec<u8> {
        (0..DISK_BYTES).map(|offset| offset as u8).collect()
    }

    /// A controller with `disk` in drive 0, selected, with the head loaded.
    fn ready_dcdd(disk: Vec<u8>) -> Dcdd {
        let mut dcdd = Dcdd::new();
        dcdd.insert(0, disk);
        dcdd.output(SELECT_PORT, 0x00);
        dcdd.output(COMMAND_PORT, HEAD_LOAD_COMMAND);
        dcdd
    }

    /// Reads the sector position until sector `sector` comes up.
    fn wait_for_sector(dcdd: &mut Dcdd, sector: u8) {
        while dcdd.input(COMMAND_PORT) != Some(SECTOR_UNUSED_BITS | sector << 1) {}
    }

    #[test]
    fn status_without_drive_has_every_bit_set() {
        let mut dcdd = Dcdd::new();

        let status = dcdd.input(SELECT_PORT);

        assert_eq!(status, Some(0xFF));
    }

    #[test]
    fn selecting_empty_drive_leaves_controller_disabled() {
        let mut dcdd = Dcdd::new();
        dcdd.insert(0, numbered_disk());

        dcdd.output(SELECT_PORT, 0x01);

        assert_eq!(dcdd.input(SELECT_PORT), Some(0xFF));
    }

    #[test]
    fn selecting_drive_with_disk_enables_it_on_track_0() {
        let mut dcdd = Dcdd::new();
        dcdd.insert(0, numbered_disk());

        dcdd.output(SELECT_PORT, 0x00);

        let status = dcdd.input(SELECT_PORT).unwrap();
        assert_eq!(status & (ENABLED_BIT | MOVE_OK_BIT | TRACK_0_BIT), 0);
        assert_eq!(status & HEAD_LOADED_BIT, HEAD_LOADED_BIT); // head not loaded
    }

    #[test]
    fn deselect_disables_controller() {
        let mut dcdd = ready_dcdd(numbered_disk());

        dcdd.output(SELECT_PORT, DESELECT);

        assert_eq!(dcdd.input(SELECT_PORT), Some(0xFF));
    }

    #[test]
    fn head_load_reports_loaded_head_and_waiting_byte() {
        let dcdd = &mut ready_dcdd(numbered_disk());

        let status = dcdd.input(SELECT_PORT).unwrap();

        assert_eq!(status & (HEAD_LOADED_BIT | READ_READY_BIT), 0);
    }

    #[test]
    fn step_in_leaves_track_0_and_step_out_returns() {
        let mut dcdd = ready_dcdd(numbered_disk());

        dcdd.output(COMMAND_PORT, STEP_IN_COMMAND);
        let after_step_in = dcdd.input(SELECT_PORT).unwrap();
        dcdd.output(COMMAND_PORT, STEP_OUT_COMMAND);
        let after_step_out = dcdd.input(SELECT_PORT).unwrap();

        assert_eq!(after_step_in & TRACK_0_BIT, TRACK_0_BIT);
        assert_eq!(after_step_out & TRACK_0_BIT, 0);
    }

    #[test]
    fn step_out_stops_at_track_0() {
        let mut dcdd = ready_dcdd(numbered_disk());

        dcdd.output(COMMAND_PORT, STEP_OUT_COMMAND);
        dcdd.output(COMMAND_PORT, STEP_IN_COMMAND);

        assert_eq!(dcdd.input(SELECT_PORT).unwrap() & TRACK_0_BIT, TRACK_0_BIT);
    }

    #[test]
    fn sector_position_counts_up_and_wraps() {
        let mut dcdd = ready_dcdd(numbered_disk());

        let positions: Vec<u8> = (0..SECTORS + 1)
            .map(|_| dcdd.input(COMMAND_PORT).unwrap() >> 1 & 0x1F)
            .collect();

        assert_eq!(positions[0], 0);
        assert_eq!(positions[31], 31);
        assert_eq!(positions[32], 0);
    }

    #[test]
    fn sector_position_without_loaded_head_has_every_bit_set() {
        let mut dcdd = Dcdd::new();
        dcdd.insert(0, numbered_disk());
        dcdd.output(SELECT_PORT, 0x00);

        let position = dcdd.input(COMMAND_PORT);

        assert_eq!(position, Some(0xFF));
    }

    #[test]
    fn read_returns_bytes_of_sector_under_head() {
        let mut dcdd = ready_dcdd(numbered_disk());
        dcdd.output(COMMAND_PORT, STEP_IN_COMMAND); // track 1
        wait_for_sector(&mut dcdd, 2);

        let bytes = [dcdd.input(DATA_PORT), dcdd.input(DATA_PORT)];

        let offset = (SECTORS + 2) * SECTOR_BYTES;
        assert_eq!(bytes, [Some(offset as u8), Some((offset + 1) as u8)]);
    }

    #[test]
    fn write_stores_bytes_in_sector_under_head() {
        let mut dcdd = ready_dcdd(vec![0; DISK_BYTES]);
        wait_for_sector(&mut dcdd, 3);
        dcdd.output(COMMAND_PORT, WRITE_ENABLE_COMMAND);

        dcdd.output(DATA_PORT, 0x11);
        dcdd.output(DATA_PORT, 0x22);

        let offset = 3 * SECTOR_BYTES;
        let disk = dcdd.disk(0).unwrap();
        assert_eq!(disk[offset..offset + 3], [0x11, 0x22, 0x00]);
    }

    #[test]
    fn write_without_write_enable_changes_nothing() {
        let mut dcdd = ready_dcdd(vec![0; DISK_BYTES]);
        wait_for_sector(&mut dcdd, 0);

        dcdd.output(DATA_PORT, 0x11);

        assert!(dcdd.disk(0).unwrap().iter().all(|&byte| byte == 0));
    }
}
