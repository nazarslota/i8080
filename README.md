# i8080

An Intel 8080 emulator written in Rust, built to run the MITS Altair 8800:
from Altair BASIC in your terminal to a real CP/M booted from a floppy image.

> **Status:** work in progress. The CPU core passes all four diagnostic
> programs, and the Altair runs Altair BASIC in your terminal, loaded from
> its original paper tape. Floppy disks and CP/M come next.

## Roadmap

- [x] **CPU core.** The full 8080 instruction set, verified with the classic
  diagnostic programs (TST8080, 8080PRE, CPUTEST, 8080EXM).
- [x] **Altair BASIC.** An Altair with memory and an 88-2SIO serial board,
  wired to your terminal.
- [ ] **Floppy disks.** The 88-DCDD controller, enough to boot Altair Disk
  BASIC, Altair DOS and CP/M 2.2.
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
instruction set of the 8080 and
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
cargo run --release -- --tape "roms/8K BASIC Ver 4-0.tap" --basic 8k-4.0
```

The emulator toggles in the MITS bootstrap loader for that BASIC, and the
loader reads the tape through the 88-2SIO serial card. `--basic` takes
`4k-4.0`, `8k-4.0`, `extended-4.0` or `extended-4.1`. Press Ctrl+] to quit.

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

You can also load a memory image at 0000H with
`cargo run --release -- <image.bin>`. Run `altair --help` for every option.

## ROMs and disk images

This repository contains no copyrighted software. You put the CPU
diagnostic programs and the BASIC tapes into [`roms/`](roms), which git
ignores.
See [roms/README.md](roms/README.md) for what goes there.

`cargo test` runs TST8080, 8080PRE and CPUTEST. 8080EXM takes minutes in a
debug build, so you run it on its own:

```sh
cargo test --release -p i8080 --test diagnostics -- --ignored
```

## References

- [Intel 8080 Microcomputer Systems User's Manual](https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf)
  (September 1975): the instruction set with cycle counts, plus the pinout
  and bus timing.
- [8080 opcode table](https://pastraiser.com/cpu/i8080/i8080_opcodes.html):
  all 256 opcodes, including the undocumented copies of `NOP`, `JMP`,
  `CALL` and `RET` that the Intel manual leaves out.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in this work, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
