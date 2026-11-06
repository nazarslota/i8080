//! Checks the execution trace lines.

use i8080::{Flags, Registers, TraceLine};

#[test]
fn line_shows_registers_before_instruction() {
    let memory = [0x7E]; // MOV A,M
    let registers = Registers {
        a: 0x12,
        b: 0x00,
        c: 0xEB,
        d: 0x2C,
        e: 0x00,
        h: 0xFF,
        l: 0x13,
        sp: 0x4C00,
        pc: 0x0000,
        flags: Flags::default(),
    };

    let line = TraceLine::new(registers, |address| memory[usize::from(address)]);

    assert_eq!(
        line.to_string(),
        "0000  7E        MOV A,M        12 00EB 2C00 FF13 4C00 .....",
    );
}

#[test]
fn line_shows_set_flags_as_letters() {
    let memory = [0x00]; // NOP
    let registers = Registers {
        flags: Flags {
            sign: true,
            zero: false,
            aux_carry: true,
            parity: false,
            carry: true,
        },
        ..Registers::default()
    };

    let line = TraceLine::new(registers, |address| memory[usize::from(address)]);

    assert!(line.to_string().ends_with(" S.A.C"));
}

#[test]
fn new_reads_from_pc_and_wraps_at_top_of_memory() {
    let mut memory = [0u8; 0x10000];
    memory[0xFFFF] = 0xC3; // JMP 1234H, with the address bytes at 0000H
    memory[0x0000..0x0002].copy_from_slice(&[0x34, 0x12]);
    let registers = Registers {
        pc: 0xFFFF,
        ..Registers::default()
    };

    let line = TraceLine::new(registers, |address| memory[usize::from(address)]);

    assert!(
        line.to_string()
            .starts_with("FFFF  C3 34 12  JMP 1234H      ")
    );
}

#[test]
fn header_names_columns_above_values() {
    let memory = [0x31, 0xFF, 0xFF]; // LXI SP,0FFFFH, the longest instruction
    let line = TraceLine::new(Registers::default(), |address| memory[usize::from(address)]);

    let text = line.to_string();

    assert_eq!(text.len(), TraceLine::HEADER.len());
    for name in [
        "BYTES",
        "INSTRUCTION",
        "A ",
        "BC",
        "DE",
        "HL",
        "SP",
        "SZAPC",
    ] {
        let column = TraceLine::HEADER.find(name).unwrap();
        assert_ne!(text.as_bytes()[column], b' ', "{name}");
        assert_eq!(text.as_bytes()[column - 1], b' ', "{name}");
    }
}
