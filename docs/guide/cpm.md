# CP/M 2.2

CP/M 2.2b boots from its original floppy image, with the MITS disk boot
loader in a PROM. [roms/README.md](../../roms/README.md) says where to get
`cpm22b23-56k.dsk` and `DBL.HEX`.

The emulator writes each change to the image file at once, so copy the
image first and boot the copy:

```sh
cp roms/cpm22b23-56k.dsk a.dsk
cargo run --release -- cpm a.dsk
```

The emulator installs `roms/DBL.HEX` as a PROM at FF00H, inserts `a.dsk` in
drive A: and starts the CPU at FF00H. `--prom` names another loader.
[boot.md](../boot.md) shows how CP/M gets from the disk to `A>`. Press
Ctrl+] to quit.

The files you create in CP/M stay in `a.dsk`. Git ignores it, and you start
over with a fresh `cp`.

## Commands

| Command           | What it does                                     |
|-------------------|--------------------------------------------------|
| `DIR`             | lists the files on the disk                      |
| `TYPE IOBYTE.TXT` | shows a text file                                |
| `STAT`            | shows the free space                             |
| `MBASIC`          | starts Microsoft BASIC; `SYSTEM` returns to `A>` |
| `ERA X.COM`       | erases a file                                    |
| `B:`              | switches to drive B:                             |

Ctrl+C at `A>` reloads CP/M from the disk. Backspace erases a character:
the emulator turns it into Ctrl+H, the key that CP/M and MBASIC erase on.

## Your own programs

The CP/M disk has 26 KB free. Make an empty disk for drive B:, copy your
`.COM` files onto it and boot with both disks:

```sh
cargo run --release -- disk format b.dsk
cargo run --release -- disk put b.dsk roms/TST8080.COM roms/CPUTEST.COM
cargo run --release -- disk ls b.dsk
cargo run --release -- cpm a.dsk b.dsk
```

```text
A>B:TST8080
MICROCOSM ASSOCIATES 8080/8085 CPU DIAGNOSTIC
 VERSION 1.0  (C) 1980

 CPU IS OPERATIONAL
```

| Command       | What it does                                                        |
|---------------|---------------------------------------------------------------------|
| `disk format` | makes an empty 296 KB disk; it refuses to overwrite a file          |
| `disk put`    | copies files, names in capitals, in place of files with those names |
| `disk ls`     | lists the files and the free space                                  |

You can name up to four disks, for A: to D:. A disk from `format` holds no
system, so CP/M can't boot from it. On a drive without a disk, CP/M prints
`Bdos Err On C: Bad Sector`, and Ctrl+C brings back `A>`.

## C programs

z88dk cross-compiles C for CP/M on your Mac. It has no Homebrew formula,
so you run its Docker image. Start Docker Desktop and fetch the image once:

```sh
docker pull z88dk/z88dk
```

C programs live in [`examples/c`](../../examples/c), each in a folder of
its own: `examples/c/main/main.c`. In `examples/c`, `make run` compiles
`main.c`, copies it to drive B: and boots CP/M with `a.dsk` and `b.dsk`
from the root of the repository. It makes either disk first if it is
missing. At `A>`, run the program from drive B: with `B:MAIN`.

```sh
cd examples/c
make run
```

| Command                 | What it does                                   |
|-------------------------|------------------------------------------------|
| `make`                  | builds every program that changed              |
| `make run`              | builds `main.c`, puts it on `b.dsk`, boots     |
| `make run PROGRAM=name` | the same for `name/name.c`                     |
| `make clean`            | removes the built `.COM` files                 |

The [Makefile](../../examples/c/Makefile) calls z88dk as
`zcc +cpm -clib=8080 -lm main.c -o main.COM`. `+cpm` builds a `.COM` file
for CP/M, and `-clib=8080` keeps the code to 8080 instructions: without it
z88dk writes Z80 code, which the 8080 can't run. `-lm` links the math
library for `float`, `double`, `pow` and `%f`; a program without them
stays the same size. The library turns `\n` into the CR LF that a
terminal expects. `put` replaces the old file on `b.dsk`, so `make run`
always boots the latest build.

## Assembly programs

CP/M brings its own 8080 assembler, `ASM.COM`, and the loader `LOAD.COM`
on drive A:. Assembly programs live in [`examples/asm`](../../examples/asm),
each in a folder of its own: `examples/asm/main/main.asm`. In
`examples/asm`, `make run` puts `main.asm` on drive B: and boots CP/M.
Then assemble, load and run it at `A>`:

```sh
cd examples/asm
make run
```

```text
A>B:
B>A:ASM MAIN
CP/M ASSEMBLER - VER 2.0
011A
000H USE FACTOR
END OF ASSEMBLY

B>A:LOAD MAIN

FIRST ADDRESS 0100
LAST  ADDRESS 0119
BYTES READ    001A
RECORDS WRITTEN 01

B>MAIN
HELLO FROM ASM
```

`ASM` writes `MAIN.HEX`, the program in Intel HEX, and `MAIN.PRN`, a
listing with the address and the bytes of each line: `TYPE MAIN.PRN`
shows it. `LOAD` turns the HEX file into `MAIN.COM`, 26 bytes here.
`make run PROGRAM=name` does the same for `name/name.asm`.

CP/M ends each line of text with CR LF, and `ASM` reads a file with bare
LF as empty. So the Makefile copies the source to `build/` with CR LF
before it puts it on the disk, and you edit it as any text file.

## Speed

The emulator runs the CPU at the 2 MHz of the real Altair, so CPUTEST
computes for minutes. Add `--fast` to run it as fast as your computer can:

```sh
cargo run --release -- --fast cpm a.dsk b.dsk
```
