# Architecture

This document explains how the emulator fits together and why we built it
this way. Read it before you change code. Sections marked **(planned)**
describe designs we agreed on but haven't written yet.

## The big picture

A computer has a processor, memory and devices such as a terminal or a floppy
drive. The processor reads instructions from memory one after another and
talks to devices through I/O ports.

The emulator copies that split. `i8080` plays the processor. A machine crate
plays everything around it. `app` connects the machine to your real computer.

```
                ┌──────────────────────────┐
                │ app                      │  your terminal, files,
                └────────────┬─────────────┘  command line
                             │
                ┌────────────┴─────────────┐
                │ altair                   │  memory, serial card,
                └────────────┬─────────────┘  floppy controller
                             │
                ┌────────────┴─────────────┐
                │ i8080                    │  the processor
                └──────────────────────────┘
```

## Crates

| Crate    | Plays the role of                    | Depends on | Touches the host                            |
|----------|--------------------------------------|------------|---------------------------------------------|
| `i8080`  | the Intel 8080 processor             | nothing    | no                                          |
| `altair` | the MITS Altair 8800 computer        | `i8080`    | no                                          |
| `app`    | you, sitting at the terminal         | `altair`   | yes                                         |

Arrows in the diagram point down. A crate never depends on a crate above it,
and the compiler enforces this: `i8080/Cargo.toml` lists no dependencies, so
code in `i8080` can't call `altair`.

## Three rules

### 1. The processor knows nothing about the machine

The 8080 has two address spaces: 64 KB of memory and 256 I/O ports. The CPU
reaches them through two traits, `MemoryBus` and `IoBus`. Each machine crate
implements both and maps every address and port to RAM, ROM, a device
register or nothing at all. To add a new machine, you implement the two
traits and leave `i8080` alone.

### 2. We emulate hardware and run original software

`altair` copies the Altair's circuits: memory boards, the serial card, the
floppy controller. Altair BASIC, Altair DOS and CP/M run unmodified from ROM
and disk images, byte for byte the programs people ran in the 1970s. We write
none of them.

The diagnostics test bends this rule. `crates/i8080/tests/diagnostics.rs`
fakes the two CP/M system calls that the diagnostic programs make, so you can
run them without emulated hardware. No library crate contains this code.

### 3. Libraries do no I/O

`i8080` and `altair` never open files, read the keyboard or print. You
hand them bytes and read bytes back, so tests can run them without a terminal
or a disk. All contact with your real computer lives in `app`.

## Walkthrough: BASIC prints a letter (planned)

Follow one character through the layers. Altair BASIC wants to print `A` on
the terminal:

1. BASIC loads `0x41` (the ASCII code for `A`) into register A and executes
   `OUT 0x11`.
2. `i8080` decodes the instruction and calls `io.output(0x11, 0x41)`.
3. The `altair` backplane finds the card that claims port `0x11`: the 88-2SIO
   serial card. The card puts the byte in its transmit buffer.
4. After each batch of instructions, `app` takes the bytes from the serial
   card and writes them to your terminal.

A keystroke travels the other way. `app` hands the key to the serial card,
and the card sets the "data ready" bit in its status port. BASIC reads that
port in a loop, sees the bit and reads the key with `IN 0x11`.

## CPU core (in progress)

- **Two buses, one per address space.** `MemoryBus` covers memory and `IoBus`
  covers ports, so code that needs only memory never sees ports. Machines and
  tests without devices pass `Unconnected`, whose ports read `0xFF` like
  unconnected ports on real hardware.
- **The machine lends the buses to the CPU.** `step` borrows the memory bus
  and the I/O bus for one instruction and keeps neither. We need no shared
  ownership (`Rc<RefCell<…>>`), and Rust resolves every bus call at compile
  time. A machine that keeps memory and devices in separate fields can lend
  both at once.
- **Decode first, then execute.** `step` turns the bytes at `PC` into an
  `Instruction` value and then executes that value. Decoding depends on the
  bytes alone, so the same code will drive the disassembler (`impl Display
  for Instruction`) and execution traces.
- **The execute step is a dispatch table.** The `match` in `step` has one
  line per instruction. An instruction with any logic gets its own method,
  named after it (`mvi`, `mov`), which does the work and returns the cycle
  count.
- **Decoding can't fail.** The 8080 treats all 256 opcodes as valid. The 12
  undocumented ones duplicate documented ones: `0x08` acts as `NOP`, `0xCB` as
  `JMP`. `step` returns no `Result`.
- **Each step reports clock cycles.** A machine needs the count to run at the
  original speed and to time interrupts. Conditional `CALL` and `RET` take
  longer when the condition holds, so the count depends on the flags too.

What works and what's left:

- [x] All 256 opcodes decode and execute, with cycle counts from the Intel
  manual. [instructions.md](instructions.md) describes each instruction.
- [ ] **Interrupts.** `EI` and `DI` switch the interrupt enable, but the CPU
  can't accept an interrupt yet: a device can't make it run an `RST`, and
  `EI` doesn't wait one instruction before interrupts take effect. The
  Altair runs without interrupts. Space Invaders needs them.
- [x] **Diagnostics.** TST8080, 8080PRE, CPUTEST and 8080EXM pass. They
  test every instruction and flag against a real 8080, and
  `crates/i8080/tests/diagnostics.rs` runs them. The unit tests check the
  CPU against the manual; the diagnostics check it against the chip.
- [ ] **Disassembler.** `impl Display for Instruction`, so a trace shows
  `MVI A,42H` and not raw bytes. It helps to find the instruction where a
  diagnostic fails.

## Altair (planned)

The real Altair is a box with a backplane: a row of 100-pin S-100 slots. You
plug in cards for memory, for a serial port to a terminal, for a floppy
controller. The code follows the same shape:

- Each card implements the `Device` trait and claims a range of memory
  addresses or I/O ports.
- The `Backplane` implements `MemoryBus` and `IoBus`. On each access it finds
  the card that claims the address or port and forwards the access to it,
  like the address decoder on each real card. An address that no card claims
  reads as `0xFF`, like an empty slot on the real bus.

To add hardware, you write a new card and plug it into the backplane. The CPU
and the other cards stay as they are.

## Glossary

**Bus.** The wires between the processor and everything else. In code, two
traits: `MemoryBus` reads and writes memory bytes, `IoBus` reads and writes
ports.

**I/O port.** A numbered channel, 0 to 255 on the 8080. The processor talks
to devices through ports with the `IN` and `OUT` instructions.

**CP/M.** The most popular operating system for 8080 and Z80 computers in the
late 1970s. It runs programs stored as `.COM` files.

**BDOS (Basic Disk Operating System).** The part of CP/M that programs call
for files and the console, with `CALL 0005H`. It calls the BIOS, which drives
the hardware.

**TPA (Transient Program Area).** The memory where CP/M loads and runs a
program, from `0100H` up to CP/M itself. A program stays there only while it
runs; the next one replaces it.

**HLE (high-level emulation).** Emulating what an operating system does
instead of the hardware it runs on. The diagnostics test uses it for two
CP/M system calls.

**S-100.** The Altair's 100-pin expansion bus. Other manufacturers copied it,
and it became an industry standard.

**Host.** Your real computer, the one running the emulator.
