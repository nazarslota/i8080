//! Checks the disassembler on all 256 opcodes.

use i8080::disassemble;

/// The operand bytes after each opcode: `34H` for an 8-bit constant and
/// `1234H` for a 16-bit one.
const OPERAND_BYTES: [u8; 2] = [0x34, 0x12];

/// What each opcode prints, in opcode order. The rows hold eight opcodes, so
/// the first one ends in 0 or 8.
#[rustfmt::skip]
const LISTING: [&str; 256] = [
    // 00H
    "NOP", "LXI B,1234H", "STAX B", "INX B", "INR B", "DCR B", "MVI B,34H", "RLC",
    "NOP", "DAD B", "LDAX B", "DCX B", "INR C", "DCR C", "MVI C,34H", "RRC",
    // 10H
    "NOP", "LXI D,1234H", "STAX D", "INX D", "INR D", "DCR D", "MVI D,34H", "RAL",
    "NOP", "DAD D", "LDAX D", "DCX D", "INR E", "DCR E", "MVI E,34H", "RAR",
    // 20H
    "NOP", "LXI H,1234H", "SHLD 1234H", "INX H", "INR H", "DCR H", "MVI H,34H", "DAA",
    "NOP", "DAD H", "LHLD 1234H", "DCX H", "INR L", "DCR L", "MVI L,34H", "CMA",
    // 30H
    "NOP", "LXI SP,1234H", "STA 1234H", "INX SP", "INR M", "DCR M", "MVI M,34H", "STC",
    "NOP", "DAD SP", "LDA 1234H", "DCX SP", "INR A", "DCR A", "MVI A,34H", "CMC",
    // 40H
    "MOV B,B", "MOV B,C", "MOV B,D", "MOV B,E", "MOV B,H", "MOV B,L", "MOV B,M", "MOV B,A",
    "MOV C,B", "MOV C,C", "MOV C,D", "MOV C,E", "MOV C,H", "MOV C,L", "MOV C,M", "MOV C,A",
    // 50H
    "MOV D,B", "MOV D,C", "MOV D,D", "MOV D,E", "MOV D,H", "MOV D,L", "MOV D,M", "MOV D,A",
    "MOV E,B", "MOV E,C", "MOV E,D", "MOV E,E", "MOV E,H", "MOV E,L", "MOV E,M", "MOV E,A",
    // 60H
    "MOV H,B", "MOV H,C", "MOV H,D", "MOV H,E", "MOV H,H", "MOV H,L", "MOV H,M", "MOV H,A",
    "MOV L,B", "MOV L,C", "MOV L,D", "MOV L,E", "MOV L,H", "MOV L,L", "MOV L,M", "MOV L,A",
    // 70H
    "MOV M,B", "MOV M,C", "MOV M,D", "MOV M,E", "MOV M,H", "MOV M,L", "HLT", "MOV M,A",
    "MOV A,B", "MOV A,C", "MOV A,D", "MOV A,E", "MOV A,H", "MOV A,L", "MOV A,M", "MOV A,A",
    // 80H
    "ADD B", "ADD C", "ADD D", "ADD E", "ADD H", "ADD L", "ADD M", "ADD A",
    "ADC B", "ADC C", "ADC D", "ADC E", "ADC H", "ADC L", "ADC M", "ADC A",
    // 90H
    "SUB B", "SUB C", "SUB D", "SUB E", "SUB H", "SUB L", "SUB M", "SUB A",
    "SBB B", "SBB C", "SBB D", "SBB E", "SBB H", "SBB L", "SBB M", "SBB A",
    // A0H
    "ANA B", "ANA C", "ANA D", "ANA E", "ANA H", "ANA L", "ANA M", "ANA A",
    "XRA B", "XRA C", "XRA D", "XRA E", "XRA H", "XRA L", "XRA M", "XRA A",
    // B0H
    "ORA B", "ORA C", "ORA D", "ORA E", "ORA H", "ORA L", "ORA M", "ORA A",
    "CMP B", "CMP C", "CMP D", "CMP E", "CMP H", "CMP L", "CMP M", "CMP A",
    // C0H
    "RNZ", "POP B", "JNZ 1234H", "JMP 1234H", "CNZ 1234H", "PUSH B", "ADI 34H", "RST 0",
    "RZ", "RET", "JZ 1234H", "JMP 1234H", "CZ 1234H", "CALL 1234H", "ACI 34H", "RST 1",
    // D0H
    "RNC", "POP D", "JNC 1234H", "OUT 34H", "CNC 1234H", "PUSH D", "SUI 34H", "RST 2",
    "RC", "RET", "JC 1234H", "IN 34H", "CC 1234H", "CALL 1234H", "SBI 34H", "RST 3",
    // E0H
    "RPO", "POP H", "JPO 1234H", "XTHL", "CPO 1234H", "PUSH H", "ANI 34H", "RST 4",
    "RPE", "PCHL", "JPE 1234H", "XCHG", "CPE 1234H", "CALL 1234H", "XRI 34H", "RST 5",
    // F0H
    "RP", "POP PSW", "JP 1234H", "DI", "CP 1234H", "PUSH PSW", "ORI 34H", "RST 6",
    "RM", "SPHL", "JM 1234H", "EI", "CM 1234H", "CALL 1234H", "CPI 34H", "RST 7",
];

/// Disassembles `bytes` and returns the text and the bytes the instruction
/// took.
fn line(bytes: &[u8]) -> (String, Vec<u8>) {
    let mut bytes = bytes.iter().copied();

    let disassembled = disassemble(|| bytes.next().unwrap_or(0));

    (disassembled.to_string(), disassembled.bytes().to_vec())
}

#[test]
fn every_opcode_prints_as_the_listing_says() {
    for (opcode, expected) in (0..=u8::MAX).zip(LISTING) {
        let bytes = [opcode, OPERAND_BYTES[0], OPERAND_BYTES[1]];

        let (text, _) = line(&bytes);

        assert_eq!(text, expected, "opcode {opcode:02X}H");
    }
}

#[test]
fn every_opcode_takes_as_many_bytes_as_its_operand_needs() {
    for (opcode, listing) in (0..=u8::MAX).zip(LISTING) {
        let bytes = [opcode, OPERAND_BYTES[0], OPERAND_BYTES[1]];
        let expected = if listing.contains("1234H") {
            3
        } else if listing.contains("34H") {
            2
        } else {
            1
        };

        let (_, taken) = line(&bytes);

        assert_eq!(taken, bytes[..expected], "opcode {opcode:02X}H");
    }
}

#[test]
fn numbers_starting_with_a_letter_get_a_leading_zero() {
    let cases: [(&[u8], &str); 8] = [
        (&[0x3E, 0x9F], "MVI A,9FH"),
        (&[0x3E, 0xA0], "MVI A,0A0H"),
        (&[0xDB, 0xFF], "IN 0FFH"),
        (&[0xD3, 0x00], "OUT 00H"),
        (&[0xC3, 0xFF, 0x9F], "JMP 9FFFH"),
        (&[0xC3, 0x00, 0xA0], "JMP 0A000H"),
        (&[0xCD, 0xFF, 0xFF], "CALL 0FFFFH"),
        (&[0x21, 0x00, 0x00], "LXI H,0000H"),
    ];

    for (bytes, expected) in cases {
        let (text, _) = line(bytes);

        assert_eq!(text, expected);
    }
}

#[test]
fn undocumented_opcode_keeps_its_original_byte() {
    let (text, bytes) = line(&[0xCB, 0x00, 0x01]);

    assert_eq!(text, "JMP 0100H");
    assert_eq!(bytes, [0xCB, 0x00, 0x01]);
}
