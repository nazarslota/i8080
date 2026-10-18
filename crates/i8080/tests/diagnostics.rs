//! Runs the classic 8080 diagnostic programs and checks that they pass.
//!
//! The programs check every instruction and flag against a real 8080. People
//! wrote them for CP/M, so this file fakes the two calls they make to the BDOS
//! (Basic Disk Operating System): print a character and print a string. It
//! emulates nothing else.
//!
//! The programs go into `roms/`; see `roms/README.md`. 8080EXM takes minutes
//! in a debug build, so run it with
//! `cargo test --release -p i8080 --test diagnostics -- --ignored`.
//!
//! # Memory layout
//!
//! Real CP/M keeps its own code at the bottom and the top of memory and gives
//! the middle to the program. The test writes only the bytes that the
//! programs use:
//!
//! ```text
//! 0000H  00H          WARM_BOOT_ADDRESS  the program jumps here to exit
//! 0005H  C3H 00H FEH  BDOS_CALL_ADDRESS  JMP FE00H, the program calls it
//! 0100H  the program  TPA_ADDRESS        PC starts here
//!  ...   free memory                     the program's data and stack
//! FE00H  C9H          BDOS_CODE_ADDRESS  RET
//! ```
//!
//! The bytes at `0006H` and `0007H` are the address part of the `JMP`. CP/M
//! programs read the word there to find the top of their memory, so they put
//! the stack below `FE00H`.
//!
//! # One system call
//!
//! A CP/M program prints text by calling the BDOS. Take a program that
//! prints the letter `A` with BDOS function 2 ([`CONSOLE_OUTPUT_FUNCTION`]):
//!
//! ```text
//! 0100H  0EH 02H      MVI C,02H   function 2: print one character
//! 0102H  1EH 41H      MVI E,41H   the character: 41H is 'A' in ASCII
//! 0104H  CDH 05H 00H  CALL 0005H  call the BDOS
//! 0107H  C3H 00H 00H  JMP 0000H   exit
//! ```
//!
//! The CPU and the test run it like this:
//!
//! 1. `CALL 0005H` pushes the return address `0107H` and jumps to `0005H`.
//! 2. The CPU executes `JMP FE00H`.
//! 3. The loop in [`run_diagnostic`] sees `PC = FE00H` before the next step
//!    and calls [`call_bdos`]. It reads 2 from `C` and appends `E`, the
//!    letter `A`, to the output.
//! 4. The CPU executes `RET` at `FE00H`, pops `0107H` and continues there.
//! 5. The CPU executes `JMP 0000H`. The loop sees `PC = 0000H` and stops
//!    before the CPU executes anything there. The output holds `A`.

use std::path::Path;

use i8080::{Bus, Cpu, Registers, Unconnected};

/// CP/M loads a program at this address and starts it there.
///
/// TPA stands for Transient Program Area: the memory between the first page
/// and CP/M itself. A program stays there only while it runs; when it exits,
/// CP/M loads the next one at the same address. The first page
/// (`0000H`–`00FFH`) belongs to CP/M: it holds the jumps at
/// [`WARM_BOOT_ADDRESS`] and [`BDOS_CALL_ADDRESS`] and the buffer for the
/// command line.
const TPA_ADDRESS: u16 = 0x0100;

/// A jump to this address ends the program.
///
/// In real CP/M the address holds `JMP` to the BIOS warm boot code. The BIOS
/// reloads CP/M from the disk and shows the `A>` prompt. The test leaves the
/// byte at 0, a `NOP`: the loop in [`run_diagnostic`] stops when `PC` gets
/// here, and the CPU never executes it.
const WARM_BOOT_ADDRESS: u16 = 0x0000;

/// Programs call this address to ask CP/M for a service. The program puts
/// the function number in register `C`, its argument in `E` or `DE`, and
/// executes `CALL 0005H`. The address sits in the `CALL` itself (bytes
/// `CDH 05H 00H`), not in a register.
const BDOS_CALL_ADDRESS: u16 = 0x0005;

/// Where the BDOS code starts: the jump at [`BDOS_CALL_ADDRESS`] leads here.
/// Real CP/M keeps kilobytes of code here; the test keeps one `RET`.
///
/// Programs read this address from `0006H` and treat it as the top of their
/// memory. Any address above the program works; `FE00H` leaves the program
/// 63 KB.
const BDOS_CODE_ADDRESS: u16 = 0xFE00;

/// The opcode of `JMP`. The test writes `JMP BDOS_CODE_ADDRESS` at
/// [`BDOS_CALL_ADDRESS`].
const JMP_OPCODE: u8 = 0xC3;
/// The opcode of `RET`. The test writes it at [`BDOS_CODE_ADDRESS`], so each
/// system call returns to the program.
const RET_OPCODE: u8 = 0xC9;

/// BDOS function 2: print the character in register `E`. The program puts 2
/// in register `C` and calls [`BDOS_CALL_ADDRESS`].
const CONSOLE_OUTPUT_FUNCTION: u8 = 2;
/// BDOS function 9: print a string from memory. The string starts at the
/// address in `DE` and ends before the first `$`. The program puts 9 in
/// register `C` and calls [`BDOS_CALL_ADDRESS`].
const PRINT_STRING_FUNCTION: u8 = 9;

#[test]
fn passes_tst8080() {
    let output = run_diagnostic("TST8080.COM");

    assert!(output.contains("CPU IS OPERATIONAL"), "{output}");
}

#[test]
fn passes_8080pre() {
    let output = run_diagnostic("8080PRE.COM");

    assert!(
        output.contains("8080 Preliminary tests complete"),
        "{output}"
    );
}

#[test]
fn passes_cputest() {
    let output = run_diagnostic("CPUTEST.COM");

    assert!(output.contains("CPU TESTS OK"), "{output}");
}

#[test]
#[ignore = "takes minutes in a debug build"]
fn passes_8080exm() {
    let output = run_diagnostic("8080EXM.COM");

    assert!(output.contains("Tests complete"), "{output}");
    assert!(!output.contains("ERROR"), "{output}");
}

/// Runs the program from `roms/` until it jumps to [`WARM_BOOT_ADDRESS`] and
/// returns what it printed.
fn run_diagnostic(name: &str) -> String {
    let program = read_program(name);

    let program_start = usize::from(TPA_ADDRESS); // CP/M loads programs here.
    let program_end = program_start + program.len(); // The program must fit below the BDOS.
    assert!(
        program_end <= usize::from(BDOS_CODE_ADDRESS),
        "{name} doesn't fit below the BDOS"
    );

    let mut memory = Box::new([0u8; 0x10000]);

    // Write the program into memory, where CP/M would load it.
    memory[program_start..program_end].copy_from_slice(&program);

    let [bdos_code_low, bdos_code_high] = BDOS_CODE_ADDRESS.to_le_bytes();
    let bdos_call = usize::from(BDOS_CALL_ADDRESS);
    memory[bdos_call..bdos_call + 3].copy_from_slice(&[JMP_OPCODE, bdos_code_low, bdos_code_high]);
    // The BDOS returns at once: the loop below does its work in Rust.
    memory[usize::from(BDOS_CODE_ADDRESS)] = RET_OPCODE;

    let mut cpu = Cpu::new();
    cpu.registers.pc = TPA_ADDRESS;
    let mut output = Vec::new();

    loop {
        match cpu.registers.pc {
            WARM_BOOT_ADDRESS => break,
            BDOS_CODE_ADDRESS => call_bdos(&cpu.registers, &memory, &mut output),
            _ => {}
        }
        cpu.step(&mut Bus {
            memory: &mut *memory,
            io: Unconnected,
        });
        assert!(!cpu.is_halted(), "{name} halted");
    }

    String::from_utf8_lossy(&output).into_owned()
}

fn read_program(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../roms")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{}: {error}. See roms/README.md", path.display()))
}

/// Performs the BDOS function that the program selected in register `C`.
fn call_bdos(registers: &Registers, memory: &[u8; 0x10000], output: &mut Vec<u8>) {
    match registers.c {
        CONSOLE_OUTPUT_FUNCTION => output.push(registers.e),
        PRINT_STRING_FUNCTION => {
            let string = memory[usize::from(registers.de())..]
                .iter()
                .take_while(|&&byte| byte != b'$');
            output.extend(string);
        }
        function => panic!("unsupported BDOS function {function}"),
    }
}
