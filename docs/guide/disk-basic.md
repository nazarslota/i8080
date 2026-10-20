# Altair Disk BASIC

Disk BASIC boots from its original floppy image through the 88-DCDD
controller and the DBL PROM. This setup supports Disk BASIC 300-5-F.
[roms/README.md](../../roms/README.md) links the image and `DBL.HEX`.

The emulator writes to the image, so boot a copy:

```sh
cp "roms/Disk BASIC 300-5-F.dsk" basic.dsk
cargo run --release -- disk-basic basic.dsk
```

Press Ctrl+] to quit. Backspace sends Ctrl+H, which Disk BASIC uses to
erase a character.

## The questions at start

Use these answers for one disk:

| Question                 | Answer                          |
|--------------------------|---------------------------------|
| `MEMORY SIZE?`           | Enter, to find the RAM          |
| `LINEPRINTER?`           | `O`, the letter, for no printer |
| `HIGHEST DISK NUMBER?`   | `0`                             |
| `HOW MANY FILES?`        | `4`                             |
| `HOW MANY RANDOM FILES?` | `4`                             |

## Save and load a program

Disk BASIC uses its own file system. Mount drive 0 before accessing files:

```text
MOUNT 0
10 PRINT "HELLO FROM DISK"
SAVE "HELLO"
FILES
NEW
LOAD "HELLO"
RUN
UNLOAD 0
```

`FILES` lists the names, and `UNLOAD 0` closes the files on that drive.
The saved program stays in `basic.dsk` after you quit. On the next boot,
use `MOUNT 0`, `LOAD "HELLO"` and `RUN` to run it. File names are
case-sensitive and have one to eight characters.

The `altair disk format`, `put` and `ls` commands work with CP/M disks,
not Disk BASIC disks. Use BASIC's commands to manage these files.

## Boot settings

The command accepts up to sixteen images for physical drives 0 to 15,
the boot disk first. Answer `HIGHEST DISK NUMBER?` with the last drive
number you supplied, such as `1` for two disks.

Disk BASIC expects sense switches `10H`: A12 up selects the 88-2SIO with
one stop bit. The command sets them for you. `--switches` overrides them,
and `--prom` names a boot loader other than `roms/DBL.HEX`.

The [Altair Clone Operator's Manual](https://altairclone.com/downloads/manuals/Altair%20Clone%20Operator%27s%20Manual.pdf),
Disk BASIC demonstration, covers the boot setup. The
[Altair BASIC Reference Manual](https://altairclone.com/downloads/manuals/BASIC%20Manual%2077.pdf),
section 5-6, describes disk files and commands.
