//! Intel HEX files: how Altair owners stored programs for PROMs. The disk
//! boot loader comes as `DBL.HEX`; the floppy boot setup reads it.
//!
//! A HEX file is text, one record per line:
//!
//! ```text
//! :10FF00002113FF11002C0EEB7E1223130DC208FFEC
//!  │ │   │ │                               └ checksum
//!  │ │   │ └ the data bytes
//!  │ │   └ the record type: 00H data, 01H end of file
//!  │ └ the address of the first data byte
//!  └ the number of data bytes
//! ```
//!
//! All bytes of a record, the checksum included, add up to 00H. A data
//! record with no bytes also ends the file: `DBL.HEX` ends this way, as the
//! HEX files from CP/M's `ASM` do.
//!
//! The format comes from Intel's [Hexadecimal Object File Format
//! Specification].
//!
//! [Hexadecimal Object File Format Specification]: https://archive.org/details/IntelHEXStandard

use super::Program;

/// A data record.
const DATA_RECORD: u8 = 0x00;
/// The end-of-file record.
const END_OF_FILE_RECORD: u8 = 0x01;
/// The bytes in front of the data: the length, two address bytes and the
/// type.
const HEADER_BYTES: usize = 4;

/// Reads the program in `text`. The records must follow each other with no
/// gaps, as in a PROM image.
pub(super) fn parse(text: &str) -> Result<Program, String> {
    let mut program: Option<Program> = None;

    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let number = index + 1;
        let record = record(line).map_err(|error| format!("line {number}: {error}"))?;
        let [length, high, low, kind] = [record[0], record[1], record[2], record[3]];
        let data = &record[HEADER_BYTES..HEADER_BYTES + usize::from(length)];
        let addr = u16::from_be_bytes([high, low]);

        match kind {
            END_OF_FILE_RECORD => break,
            DATA_RECORD if data.is_empty() => break,
            DATA_RECORD => match &mut program {
                None => {
                    program = Some(Program {
                        start: addr,
                        bytes: data.to_vec(),
                    });
                }
                Some(program) => {
                    let next = usize::from(program.start) + program.bytes.len();
                    if usize::from(addr) != next {
                        return Err(format!(
                            "line {number}: the record at {addr:04X}H leaves a gap after {next:04X}H"
                        ));
                    }
                    program.bytes.extend_from_slice(data);
                }
            },
            _ => return Err(format!("line {number}: unknown record type {kind:02X}H")),
        }
    }

    program.ok_or_else(|| "the file holds no data".to_string())
}

/// Decodes the hex digits of one record and checks its length and checksum.
fn record(line: &str) -> Result<Vec<u8>, String> {
    let digits = line
        .strip_prefix(':')
        .ok_or("a record starts with a colon")?;
    if !digits.len().is_multiple_of(2) {
        return Err("odd number of hex digits".to_string());
    }
    let bytes = (0..digits.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&digits[at..at + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "not a hex digit")?;

    let length = bytes.first().map_or(0, |&length| usize::from(length));
    if bytes.len() != HEADER_BYTES + length + 1 {
        return Err(format!("the record should hold {length} data bytes"));
    }
    if bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte)) != 0 {
        return Err("wrong checksum".to_string());
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_data_records() {
        let text = ":02FF0000C30339\n:01FF020000FE\n:00000001FF\n";

        let program = parse(text);

        assert_eq!(
            program,
            Ok(Program {
                start: 0xFF00,
                bytes: vec![0xC3, 0x03, 0x00],
            })
        );
    }

    #[test]
    fn stops_at_empty_data_record() {
        let text = ":01FF0000C33D\n:0000000000\n:01FF0100FF00\n";

        let program = parse(text);

        assert_eq!(program.map(|program| program.bytes), Ok(vec![0xC3]));
    }

    #[test]
    fn rejects_wrong_checksum() {
        let text = ":01FF0000C33E\n";

        let program = parse(text);

        assert_eq!(program, Err("line 1: wrong checksum".to_string()));
    }

    #[test]
    fn rejects_gap_between_records() {
        let text = ":01FF0000C33D\n:01FF100000F0\n";

        let program = parse(text);

        assert_eq!(
            program,
            Err("line 2: the record at FF10H leaves a gap after FF01H".to_string())
        );
    }

    #[test]
    fn rejects_short_record() {
        let text = ":02FF0000C33D\n";

        let program = parse(text);

        assert_eq!(
            program,
            Err("line 1: the record should hold 2 data bytes".to_string())
        );
    }

    #[test]
    fn rejects_line_without_colon() {
        let text = "01FF0000C33D\n";

        let program = parse(text);

        assert_eq!(
            program,
            Err("line 1: a record starts with a colon".to_string())
        );
    }

    #[test]
    fn rejects_file_without_data() {
        let text = ":00000001FF\n";

        let program = parse(text);

        assert_eq!(program, Err("the file holds no data".to_string()));
    }
}
