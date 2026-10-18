//! `altair disk`: makes, fills and lists CP/M disk images with
//! [`cpm::CpmDisk`]. The Altair stays off.
//!
//! ```
//! use app::cli::DiskCommand;
//!
//! let disk = std::env::temp_dir().join(format!("app-doc-{}-disk.dsk", std::process::id()));
//! let folder = std::env::temp_dir().join(format!("app-doc-{}-disk", std::process::id()));
//! std::fs::create_dir_all(&folder)?;
//! let file = folder.join("HELLO.COM");
//! std::fs::write(&file, [0xC9])?; // RET
//!
//! app::disk::run(&DiskCommand::Format { disk: disk.clone() })?;
//! app::disk::run(&DiskCommand::Put { disk: disk.clone(), files: vec![file.clone()] })?;
//!
//! let cpm = app::disk::cpm::CpmDisk::new(std::fs::read(&disk)?)?;
//! assert_eq!(cpm.files().len(), 1);
//! # std::fs::remove_file(&disk)?;
//! # std::fs::remove_dir_all(&folder)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod cpm;

use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use self::cpm::CpmDisk;

use crate::cli::DiskCommand;
use crate::host::{in_file, read};

/// Carries out `command`.
///
/// ```
/// use app::cli::DiskCommand;
///
/// let disk = std::env::temp_dir().join(format!("app-doc-{}-disk-run.dsk", std::process::id()));
/// app::disk::run(&DiskCommand::Format { disk: disk.clone() })?;
///
/// let again = app::disk::run(&DiskCommand::Format { disk: disk.clone() });
///
/// assert!(again.is_err()); // format never overwrites a file
/// app::disk::run(&DiskCommand::Ls { disk: disk.clone() })?; // prints "0 files, 296 KB free"
/// # std::fs::remove_file(&disk)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn run(command: &DiskCommand) -> Result<(), Box<dyn Error>> {
    match command {
        DiskCommand::Format { disk } => format(disk),
        DiskCommand::Put { disk, files } => put(disk, files),
        DiskCommand::Ls { disk } => ls(disk),
    }
}

/// Makes an empty CP/M disk at `disk`. Fails when the file exists, so it
/// can't wipe a disk with your files.
fn format(disk: &Path) -> Result<(), Box<dyn Error>> {
    File::create_new(disk)
        .and_then(|mut file| file.write_all(&CpmDisk::format().into_bytes()))
        .map_err(|error| in_file(disk, error))?;
    Ok(())
}

/// Copies `files` onto the CP/M disk at `disk`. Writes the image only when
/// all of them fit.
fn put(disk: &Path, files: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let mut cpm = open(disk)?;
    for file in files {
        let name = file
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| in_file(file, "the name is not a CP/M file name"))?;
        cpm.put(name, &read(file)?)
            .map_err(|error| in_file(file, error))?;
    }
    fs::write(disk, cpm.into_bytes()).map_err(|error| in_file(disk, error))?;
    Ok(())
}

/// Prints the files on the CP/M disk at `disk` with their sizes, then the
/// free space.
fn ls(disk: &Path) -> Result<(), Box<dyn Error>> {
    let cpm = open(disk)?;
    let files = cpm.files();
    for file in &files {
        println!("{:<12} {:>6}", file.name, file.bytes);
    }
    let noun = if files.len() == 1 { "file" } else { "files" };
    println!(
        "{} {noun}, {} KB free",
        files.len(),
        cpm.free_bytes() / 1024
    );
    Ok(())
}

/// Reads the CP/M disk at `disk`.
fn open(disk: &Path) -> Result<CpmDisk, Box<dyn Error>> {
    Ok(CpmDisk::new(read(disk)?).map_err(|error| in_file(disk, error))?)
}
