//! The PROM and disk files shared by the CP/M, Disk BASIC and DOS boot setups.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use super::{Setup, hex};
use crate::host::disk_file::DiskFile;
use crate::host::in_file;
use crate::machine::PROM_ADDRESS;

/// The bytes in the 8080's address space.
const ADDRESS_SPACE_BYTES: usize = 0x10000;

/// Opens one to `drive_count` disks and the boot PROM before terminal setup.
pub(super) fn setup(
    prom: &Path,
    disks: &[PathBuf],
    drive_count: usize,
    sense_switches: u8,
    erase_key: u8,
) -> Result<Setup, Box<dyn Error>> {
    if disks.is_empty() || disks.len() > drive_count {
        return Err(format!("expected 1 to {drive_count} disks, got {}", disks.len()).into());
    }
    let text = fs::read_to_string(prom).map_err(|error| in_file(prom, error))?;
    let loader = hex::parse(&text).map_err(|error| in_file(prom, error))?;
    if loader.start < PROM_ADDRESS {
        let error = format!(
            "the loader starts at {:04X}H, below the PROM at {PROM_ADDRESS:04X}H",
            loader.start
        );
        return Err(in_file(prom, error).into());
    }
    if usize::from(loader.start) + loader.bytes.len() > ADDRESS_SPACE_BYTES {
        return Err(in_file(prom, "the loader extends past FFFFH").into());
    }
    let disks = disks
        .iter()
        .map(|disk| DiskFile::open(disk).map_err(|error| in_file(disk, error)))
        .collect::<Result<_, _>>()?;
    Ok(Setup {
        toggle_in: None,
        start: loader.start,
        prom: Some(loader),
        disks,
        tape: Vec::new(),
        sense_switches,
        erase_key,
    })
}

#[cfg(test)]
mod tests {
    use altair::cards::{DISK_BYTES, Disk};

    use super::*;
    use crate::boot::{Program, cpm, disk_basic, dos};

    /// Test files that the fixture removes after their disk handles close.
    struct BootFiles {
        prom: PathBuf,
        disk: PathBuf,
    }

    impl BootFiles {
        fn new(name: &str) -> Result<Self, Box<dyn Error>> {
            let stem = format!("altair-floppy-{}-{name}", std::process::id());
            let files = Self {
                prom: std::env::temp_dir().join(format!("{stem}.hex")),
                disk: std::env::temp_dir().join(format!("{stem}.dsk")),
            };
            fs::write(&files.prom, ":01FF0000C33D\n:00000001FF\n")?;
            let mut bytes = vec![0; DISK_BYTES];
            bytes[0] = 0x42;
            fs::write(&files.disk, bytes)?;
            Ok(files)
        }
    }

    impl Drop for BootFiles {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.prom);
            let _ = fs::remove_file(&self.disk);
        }
    }

    #[test]
    fn cpm_setup_keeps_its_boot_settings() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("cpm")?;

        let mut setup = cpm::setup(&files.prom, std::slice::from_ref(&files.disk))?;

        assert_eq!(setup.start, 0xFF00);
        assert_eq!(setup.sense_switches, 0x00);
        assert_eq!(setup.erase_key, 0x08);
        assert!(setup.toggle_in.is_none());
        assert!(setup.tape.is_empty());
        assert_eq!(
            setup.prom,
            Some(Program {
                start: 0xFF00,
                bytes: vec![0xC3]
            })
        );
        assert_eq!(setup.disks.len(), 1);
        assert_eq!(setup.disks[0].read(0), 0x42);
        Ok(())
    }

    #[test]
    fn disk_basic_setup_selects_2sio_with_one_stop_bit() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("disk-basic")?;

        let setup = disk_basic::setup(&files.prom, std::slice::from_ref(&files.disk))?;

        assert_eq!(setup.start, 0xFF00);
        assert_eq!(setup.sense_switches, 0x10);
        assert_eq!(setup.erase_key, 0x08);
        assert!(setup.toggle_in.is_none());
        assert!(setup.tape.is_empty());
        assert_eq!(setup.disks.len(), 1);
        Ok(())
    }

    #[test]
    fn dos_setup_uses_rubout_and_serial_switches() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("dos")?;

        let setup = dos::setup(&files.prom, std::slice::from_ref(&files.disk))?;

        assert_eq!(setup.start, 0xFF00);
        assert_eq!(setup.sense_switches, 0x10);
        assert_eq!(setup.erase_key, 0x7F);
        assert!(setup.toggle_in.is_none());
        assert!(setup.tape.is_empty());
        assert_eq!(setup.disks.len(), 1);
        Ok(())
    }

    #[test]
    fn opens_disks_in_argument_order() -> Result<(), Box<dyn Error>> {
        let first = BootFiles::new("order-first")?;
        let second = BootFiles::new("order-second")?;
        let mut bytes = vec![0; DISK_BYTES];
        bytes[0] = 0x24;
        fs::write(&second.disk, bytes)?;

        let mut setup = disk_basic::setup(&first.prom, &[second.disk.clone(), first.disk.clone()])?;

        assert_eq!(setup.disks[0].read(0), 0x24);
        assert_eq!(setup.disks[1].read(0), 0x42);
        Ok(())
    }

    #[test]
    fn rejects_no_disks() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("no-disks")?;

        let basic = disk_basic::setup(&files.prom, &[]);
        let dos = dos::setup(&files.prom, &[]);

        assert!(basic.is_err());
        assert!(dos.is_err());
        Ok(())
    }

    #[test]
    fn rejects_more_drives_than_the_software_supports() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("too-many")?;

        let cpm = cpm::setup(&files.prom, &vec![files.disk.clone(); 5]);
        let basic = disk_basic::setup(&files.prom, &vec![files.disk.clone(); 17]);
        let dos = dos::setup(&files.prom, &vec![files.disk.clone(); 7]);

        assert!(cpm.is_err());
        assert!(basic.is_err());
        assert!(dos.is_err());
        Ok(())
    }

    #[test]
    fn disk_basic_opens_sixteen_drives() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("sixteen")?;

        let setup = disk_basic::setup(&files.prom, &vec![files.disk.clone(); 16])?;

        assert_eq!(setup.disks.len(), 16);
        Ok(())
    }

    #[test]
    fn dos_opens_six_drives() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("dos-six")?;

        let setup = dos::setup(&files.prom, &vec![files.disk.clone(); 6])?;

        assert_eq!(setup.disks.len(), 6);
        Ok(())
    }

    #[test]
    fn rejects_loader_below_prom() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("low-prom")?;
        fs::write(&files.prom, ":01000000C33C\n:00000001FF\n")?;

        let result = disk_basic::setup(&files.prom, std::slice::from_ref(&files.disk));

        assert!(result.is_err_and(|error| error.to_string().contains("below the PROM")));
        Ok(())
    }

    #[test]
    fn rejects_loader_past_end_of_memory() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("long-prom")?;
        fs::write(&files.prom, ":02FFFF00AABB9B\n:00000001FF\n")?;

        let result = disk_basic::setup(&files.prom, std::slice::from_ref(&files.disk));

        assert!(result.is_err_and(|error| error.to_string().contains("past FFFFH")));
        Ok(())
    }

    #[test]
    fn rejects_bad_prom_checksum() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("bad-checksum")?;
        fs::write(&files.prom, ":01FF0000C33E\n:00000001FF\n")?;

        let result = disk_basic::setup(&files.prom, std::slice::from_ref(&files.disk));

        assert!(result.is_err_and(|error| error.to_string().contains("wrong checksum")));
        Ok(())
    }

    #[test]
    fn rejects_short_disk_before_starting_the_machine() -> Result<(), Box<dyn Error>> {
        let files = BootFiles::new("short-disk")?;
        fs::write(&files.disk, [0; 100])?;

        let result = disk_basic::setup(&files.prom, std::slice::from_ref(&files.disk));

        assert!(result.is_err_and(|error| error.to_string().contains("not an 8-inch Altair disk")));
        Ok(())
    }
}
