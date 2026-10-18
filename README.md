# i8080

An Intel 8080 emulator written in Rust, built to run the MITS Altair 8800:
from Altair BASIC in your terminal to a real CP/M booted from a floppy image.

> **Status:** early work in progress. The CPU core runs every instruction
> and passes all four diagnostic programs. The Altair runs a memory image in
> your terminal; Altair BASIC comes next.

## Roadmap

- [x] **CPU core.** The full 8080 instruction set, verified with the classic
  diagnostic programs (TST8080, 8080PRE, CPUTEST, 8080EXM).
- [ ] **Altair BASIC.** An Altair with memory and an 88-2SIO serial board,
  wired to your terminal.
- [ ] **Floppy disks.** The 88-DCDD controller, enough to boot Altair Disk
  BASIC, Altair DOS and CP/M 2.2.
- [ ] Later: a window with a retro monitor, the front panel, Space Invaders.

## Workspace

| Crate                     | What it is                                                                |
|---------------------------|---------------------------------------------------------------------------|
| [`i8080`](crates/i8080)   | The CPU core. `no_std`, no dependencies, knows nothing about any machine. |
| [`altair`](crates/altair) | Altair 8800 hardware: the S-100 backplane and its cards.                  |
| [`app`](app)              | The `altair` command-line program.                                        |

See [docs/architecture.md](docs/architecture.md) for how the pieces fit
together and [docs/instructions.md](docs/instructions.md) for the 8080
instruction set and what the emulator supports so far.

## Building

Requires Rust 1.99 or newer.

```sh
cargo build
cargo test
```

## Running

```sh
cargo run --release -- <image.bin>
```

The emulator loads the memory image at 0000H, starts the CPU there and
connects the 88-2SIO serial card to your terminal. Press Ctrl+] to quit.

## ROMs and disk images

This repository contains no copyrighted software. The CPU diagnostic
programs go into [`roms/`](roms), which is ignored by git.
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
