# Altair DOS

Altair DOS 1.0 runs from its original floppy image through the DBL PROM,
the 88-DCDD controller and the 88-2SIO terminal. The emulator writes to the image,
so boot a copy:

```sh
cp roms/AltairDOS.dsk dos.dsk
cargo run --release -- dos dos.dsk
```

[roms/README.md](../../roms/README.md) links the image and `DBL.HEX`.
Press Ctrl+] to quit.

## The questions at start

Use these answers for one disk:

| Question                 | Answer                          |
|--------------------------|---------------------------------|
| `MEMORY SIZE?`           | Enter, to find the RAM          |
| `INTERRUPTS?`            | `Y`, to enable input interrupts |
| `HIGHEST DISK NUMBER?`   | `0`                             |
| `HOW MANY DISK FILES?`   | `4`                             |
| `HOW MANY RANDOM FILES?` | `4`                             |

DOS prints its monitor banner and a period (`.`) when it is ready.
Type commands in capitals. Backspace sends RUBOUT: DOS prints the deleted
characters between backslashes instead of moving the cursor back.

Wait for the full prompt before typing. Bulk input can lose characters:
the emulator does not model serial baud timing. For text sent to the
editor, [deramp recommends a 40 ms delay per character](https://deramp.com/downloads/altair/software/8_inch_floppy/Altair%20DOS/Sending%20files%20to%20Altair%20DOS.txt).

## Files and the editor

Mount drive 0 before accessing files:

```text
MNT 0
DIR 0
EDIT TEST 0
```

The editor creates `TEST` and prints a line number. Type a line of text,
press Enter, then press Enter on the empty next line. The `*` prompt
accepts editor commands. `P` prints the text; `E` saves it and returns to
DOS. `DIR 0` then lists the source file as `&TEST`.

After you quit and boot the same `dos.dsk`, read the saved text with:

```text
MNT 0
EDIT TEST 0
```

At the editor's `*` prompt, `P` prints the saved text and `Q` returns to
DOS without saving changes. Use `DSM 0` to dismount the disk before you
quit. With input interrupts enabled, Ctrl+C
interrupts a running program and returns to the monitor.

The [DOS manual](https://altairclone.com/downloads/manuals/Altair%20DOS%20User%27s%20Manual.pdf),
chapters 2 and 3, lists the monitor and editor commands. DOS marks file
types with prefixes: `#` for executable files, `&` for source files and
`$` for editor backups. The `altair disk` tools handle CP/M images, not
DOS images. Do not run DOS's `DIN` on the system disk: it erases the files.

## Boot settings

The command accepts up to six images for drives 0 to 5, the DOS system
disk first. DOS 1.0 rejects a `HIGHEST DISK NUMBER?` above `5`. For two
images, answer `1`.

The command sets sense switches `10H` for the 88-2SIO terminal with one
stop bit. `--switches` overrides them. `--prom` names a boot loader other
than `roms/DBL.HEX`.

The [Altair Clone Operator's Manual](https://altairclone.com/downloads/manuals/Altair%20Clone%20Operator%27s%20Manual.pdf),
Altair DOS demonstration, describes this boot setup.
