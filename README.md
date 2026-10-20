# i8080

An Intel 8080 emulator in Rust that runs the MITS Altair 8800 in your
terminal: Altair BASIC from its original paper tape, Disk BASIC and CP/M
from their original floppy images.

## Roadmap

- [x] **CPU core.** The full 8080 instruction set, verified with the classic
  diagnostic programs (TST8080, 8080PRE, CPUTEST, 8080EXM).
- [x] **Altair BASIC.** An Altair with memory and an 88-2SIO serial board,
  wired to your terminal.
- [x] **CP/M 2.2.** The 88-DCDD floppy controller and a PROM card with the
  MITS disk boot loader. CP/M boots, saves files to the disk image and runs
  the four CPU diagnostics.
- [x] **Disk BASIC.** Version 300-5-F boots, saves programs to its disk
  image and loads them after a restart.
- [ ] **More disks.** Altair DOS.
- [x] **CPU interrupts.** Device-supplied instructions, the delay after
  `EI` and wake-up from `HLT`.
- [x] **Altair interrupts.** Receive and transmit requests from the 88-2SIO
  serial board through PINT (`RST 7`).
- [ ] **CPU extras.** A disassembler for execution traces.
- [ ] Later: a window with a retro monitor, the front panel, Space Invaders.

## Quick start

You need Rust 1.99 or newer, and the tapes, the boot loader and the disk
image in [`roms/`](roms). This repository ships no copyrighted software;
[roms/README.md](roms/README.md) says where to get each file.

```sh
cargo run --release -- basic 8k-4.0 "roms/8K BASIC Ver 4-0.tap"   # Altair BASIC
cp "roms/Disk BASIC 300-5-F.dsk" basic.dsk
cargo run --release -- disk-basic basic.dsk                       # Disk BASIC
cp roms/cpm22b23-56k.dsk a.dsk
cargo run --release -- cpm a.dsk                                   # CP/M
cargo run --release -- image roms/hello.bin                        # a memory image
```

Press Ctrl+] to quit. `altair --help` lists every command and option.

`cargo test` runs the tests and three CPU diagnostics. The fourth, 8080EXM,
takes minutes in a debug build, so it runs on its own:

```sh
cargo test --release -p i8080 --test diagnostics -- --ignored
```

## Documentation

| Read                                                          | To learn                                                  |
|---------------------------------------------------------------|-----------------------------------------------------------|
| [guide/basic.md](docs/guide/basic.md)                         | how to run Altair BASIC                                   |
| [guide/disk-basic.md](docs/guide/disk-basic.md)               | how to boot Disk BASIC and save programs                  |
| [guide/cpm.md](docs/guide/cpm.md)                             | how to run CP/M, add disks, build C and assembly programs |
| [architecture.md](docs/architecture.md)                       | how the code splits into layers, and why                  |
| [boot.md](docs/boot.md)                                       | what happens between RUN and `OK` or `A>`                 |
| [altair/front-panel.md](docs/altair/front-panel.md)           | the Altair's switches and lights                          |
| [altair/memory-and-ports.md](docs/altair/memory-and-ports.md) | the memory map and the I/O ports                          |
| [i8080/instructions.md](docs/i8080/instructions.md)           | every 8080 instruction                                    |
| [glossary.md](docs/glossary.md)                               | PROM, BDOS, sense switches and the other terms            |

`cargo doc --open` documents the code, with an example for each public
function.

## Workspace

| Crate                     | What it is                                                                |
|---------------------------|---------------------------------------------------------------------------|
| [`i8080`](crates/i8080)   | The CPU core. `no_std`, no dependencies, knows nothing about any machine. |
| [`altair`](crates/altair) | Altair 8800 hardware: the S-100 backplane and its cards.                  |
| [`app`](app)              | The `altair` command-line program.                                        |

## References

The 8080:

- [Intel 8080 Microcomputer Systems User's Manual](https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf)
  (September 1975): the instruction set with cycle counts, plus the pinout
  and bus timing.
- [Intel 8080 Assembly Language Programming Manual](https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf):
  chapter 5 explains interrupt handling and the instruction a device supplies.
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
- [MITS 88-2SIO manual](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Altair%202SIO%20Serial%20I-O%20searchable.pdf):
  the serial board's control and status registers, interrupt enables and wiring.
- [Altair Clone Operator's Manual](https://altairclone.com/downloads/manuals/Altair%20Clone%20Operator%27s%20Manual.pdf):
  section F explains PINT and the open-bus `RST 7` without an 88-VI/RTC board.
- [Altair 8800 Monitor](https://k1.spdns.de/Develop/Projects/zxsp/zxsp.alt10/zasm/Test/8080/original/Altair8800_Monitor.z80):
  an assembly listing that defines the 88-2SIO ports and status bits.
- [Loading Basic with the 88-2SIO Board](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf):
  the serial card's ports and the bootstrap loaders for the BASIC tapes.
- [Altair Software Development using CP/M](https://deramp.com/downloads/altair/software/papertape_cassette/Checksum%20Loader/Altair%20Paper%20Tape%20Format.pdf):
  the format of an Altair paper tape.
- [Altair BASIC Reference Manual](https://altairclone.com/downloads/manuals/BASIC%20Manual%2077.pdf):
  section 5-6 describes Disk BASIC files and commands.
- [Disk BASIC images](https://deramp.com/downloads/altair/software/8_inch_floppy/BASIC/):
  the original 8-inch images, including version 300-5-F.
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
