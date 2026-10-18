# i8080

An Intel 8080 emulator written in Rust, built to run the MITS Altair 8800:
from Altair BASIC in your terminal to a real CP/M booted from a floppy image.

> **Status:** work in progress. The CPU core passes all four diagnostic
> programs, and the Altair runs Altair BASIC in your terminal, loaded from
> its original paper tape, and CP/M 2.2 boots from a floppy image.

## Roadmap

- [x] **CPU core.** The full 8080 instruction set, verified with the classic
  diagnostic programs (TST8080, 8080PRE, CPUTEST, 8080EXM).
- [x] **Altair BASIC.** An Altair with memory and an 88-2SIO serial board,
  wired to your terminal.
- [x] **CP/M 2.2.** The 88-DCDD floppy controller and a PROM card with the
  MITS disk boot loader. CP/M boots and saves files to the disk image.
- [ ] **More disks.** Run the CPU diagnostics under CP/M, then try Altair
  Disk BASIC and Altair DOS.
- [ ] **CPU extras.** Interrupts, which Space Invaders needs, and a
  disassembler for execution traces.
- [ ] Later: a window with a retro monitor, the front panel, Space Invaders.

## Workspace

| Crate                     | What it is                                                                |
|---------------------------|---------------------------------------------------------------------------|
| [`i8080`](crates/i8080)   | The CPU core. `no_std`, no dependencies, knows nothing about any machine. |
| [`altair`](crates/altair) | Altair 8800 hardware: the S-100 backplane and its cards.                  |
| [`app`](app)              | The `altair` command-line program.                                        |

See [docs/architecture.md](docs/architecture.md) for how the pieces fit
together, [docs/i8080/instructions.md](docs/i8080/instructions.md) for the
instruction set of the 8080,
[docs/altair/memory-and-ports.md](docs/altair/memory-and-ports.md) for the
memory map and the I/O ports, and
[docs/altair/front-panel.md](docs/altair/front-panel.md) for the Altair's
switches and lights.

## Building

Requires Rust 1.99 or newer.

```sh
cargo build
cargo test
```

## Running

Load Altair BASIC from its paper tape:

```sh
cargo run --release -- tape 8k-4.0 "roms/8K BASIC Ver 4-0.tap"
```

The emulator toggles in the MITS bootstrap loader for that BASIC, and the
loader reads the tape through the 88-2SIO serial card. You name the BASIC
first: `4k-4.0`, `8k-4.0`, `extended-4.0` or `extended-4.1`. Press Ctrl+]
to quit.

BASIC then asks a few questions. Keep Caps Lock on: BASIC wants capitals.

| Question                | Answer                                      |
|-------------------------|---------------------------------------------|
| `MEMORY SIZE?`          | Enter, for all the memory                   |
| `TERMINAL WIDTH?`       | Enter, for 72 columns (4K and 8K)           |
| `LINEPRINTER?`          | `O` (Extended): the emulator has no printer |
| `WANT SIN-COS-TAN-ATN?` | `Y` keeps the functions, `N` drops them     |

In 4K BASIC the last question splits into `SIN?`, `SQR?` and `RND?`.

```text
OK
10 FOR I=1 TO 3
20 PRINT "HELLO";I
30 NEXT I
RUN
HELLO 1
HELLO 2
HELLO 3
OK
```

Boot CP/M 2.2 from a floppy image. The emulator writes each change to the
image file at once, so copy the image first and boot the copy:

```sh
cp roms/cpm22b23-56k.dsk cpm.dsk
cargo run --release -- disk roms/DBL.HEX cpm.dsk
```

You name the boot loader first and the disk second. The emulator plugs a
PROM card with the MITS disk boot loader at FF00H and an 88-DCDD controller
with the disk in drive 0, then starts the CPU at FF00H. The loader reads
CP/M from the disk, and CP/M greets you with `A>`. Type `DIR` to list the
disk. The files you create in CP/M stay in `cpm.dsk`. Git ignores it, and
you start over with a fresh `cp`.

| Command           | What it does                                     |
|-------------------|--------------------------------------------------|
| `DIR`             | lists the files on the disk                      |
| `TYPE IOBYTE.TXT` | shows a text file                                |
| `STAT`            | shows the free space                             |
| `MBASIC`          | starts Microsoft BASIC; `SYSTEM` returns to `A>` |
| `ERA X.COM`       | erases a file                                    |

Ctrl+C at `A>` reloads CP/M from the disk. Backspace erases a character:
the emulator sends Ctrl+H for it, as CP/M and MBASIC expect.

To run your own program, put it on the disk before you boot, then type its
name at `A>`:

```sh
cargo run --release -- put cpm.dsk roms/TST8080.COM
cargo run --release -- disk roms/DBL.HEX cpm.dsk
```

```text
A>TST8080
MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC
 VERSION 1.0  (C) 1980

 CPU IS OPERATIONAL
```

`put` keeps the file name in capitals and replaces a file with the same
name. The CP/M disk has about 26 KB free, enough for TST8080, 8080PRE and
CPUTEST together.

CPUTEST computes for minutes at the 2 MHz of the real Altair. Add `--fast`
to run the CPU as fast as your computer can:

```sh
cargo run --release -- --fast disk roms/DBL.HEX cpm.dsk
```

You can also load a memory image at 0000H with
`cargo run --release -- image roms/hello.bin`. Run `altair --help` for every
option.

## ROMs and disk images

This repository contains no copyrighted software. You put the CPU
diagnostic programs, the BASIC tapes, the boot loader and the CP/M disk into
[`roms/`](roms), which git ignores.
See [roms/README.md](roms/README.md) for what goes there.

`cargo test` runs TST8080, 8080PRE and CPUTEST. 8080EXM takes minutes in a
debug build, so you run it on its own:

```sh
cargo test --release -p i8080 --test diagnostics -- --ignored
```

## References

The 8080:

- [Intel 8080 Microcomputer Systems User's Manual](https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf)
  (September 1975): the instruction set with cycle counts, plus the pinout
  and bus timing.
- [8080 opcode table](https://pastraiser.com/cpu/i8080/i8080_opcodes.html):
  all 256 opcodes, including the undocumented copies of `NOP`, `JMP`,
  `CALL` and `RET` that the Intel manual leaves out.
- [CPU diagnostics](https://github.com/superzazu/8080/tree/master/cpu_tests):
  TST8080, 8080PRE, CPUTEST and 8080EXM, from the superzazu/8080 emulator.
- [Intel HEX](https://archive.org/details/IntelHEXStandard): the file format
  of `DBL.HEX`.

The Altair:

- [Altair 8800 Theory of Operation](https://altairclone.com/downloads/manuals/Altair%208800%20Theory%20of%20Operation.pdf):
  the front panel and what its switches do to the CPU.
- [Loading Basic with the 88-2SIO Board](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf):
  the serial card's ports and the bootstrap loaders for the BASIC tapes.
- [Altair Software Development using CP/M](https://deramp.com/downloads/altair/software/papertape_cassette/Checksum%20Loader/Altair%20Paper%20Tape%20Format.pdf):
  the format of an Altair paper tape.
- [MITS tapes](https://www.icl1900.co.uk/unix4fun/z80pack/ftp/altair/mits-tapes/):
  the Altair BASIC tapes, on the z80pack mirror.

CP/M:

- [DBL.ASM](https://deramp.com/downloads/altair/software/roms/orginal_roms/DBL.ASM):
  the source of the MITS disk boot loader in the PROM.
- [CP/M 2.2B for the Altair](https://deramp.com/downloads/altair/software/8_inch_floppy/CPM/CPM%202.2/CPM%202.2B/):
  Mike Douglas's disk image, with `BOOT.ASM` and `BIOS.ASM`, the sources of
  its cold boot loader and BIOS.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this work, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
