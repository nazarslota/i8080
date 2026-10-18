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

## Speed

The emulator runs the CPU at the 2 MHz of the real Altair, so CPUTEST
computes for minutes. Add `--fast` to run it as fast as your computer can:

```sh
cargo run --release -- --fast cpm a.dsk b.dsk
```
