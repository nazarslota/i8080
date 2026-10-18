# ROMs and disk images

Git ignores everything in this directory except this file. The repository
ships no copyrighted software, so you supply these files yourself:

| File                                                       | Used for                                                       |
|------------------------------------------------------------|----------------------------------------------------------------|
| `TST8080.COM`, `8080PRE.COM`, `CPUTEST.COM`, `8080EXM.COM` | CPU diagnostics for `crates/i8080/tests/diagnostics.rs`        |
| `hello.bin`                                                | A memory image that prints a greeting: `altair image roms/hello.bin` |
| `4K BASIC Ver 4-0.tap`, `8K BASIC Ver 4-0.tap`             | Altair BASIC on paper tape: `altair tape 8k-4.0 …`                    |
| `Extended BASIC Ver 4-0.tap`, `Extended BASIC Ver 4-1.tap` | Extended BASIC on paper tape: `altair tape extended-4.1 …`            |
| `DBL.HEX`                                                  | The MITS disk boot loader for the PROM: `altair disk roms/DBL.HEX …`  |
| `cpm22b23-56k.dsk`                                         | CP/M 2.2b on an 8-inch floppy: `altair disk … roms/cpm22b23-56k.dsk`  |

## Where to get them

**CPU diagnostics.** Download the four `.COM` files from the
[`cpu_tests`](https://github.com/superzazu/8080/tree/master/cpu_tests)
folder of the superzazu/8080 emulator.

**BASIC tapes.** Download the four `.tap` files in the table from the
[`mits-tapes`](https://www.icl1900.co.uk/unix4fun/z80pack/ftp/altair/mits-tapes/)
folder of the z80pack mirror. Keep the file names: the examples and the run
configurations use them.

**Disk boot loader.** Download `DBL.HEX` from the
[`orginal_roms`](https://deramp.com/downloads/altair/software/roms/orginal_roms/)
folder on deramp.com. `DBL.ASM` there holds its source.

**CP/M.** Download `cpm22b23-56k.dsk` from the
[CP/M 2.2B](https://deramp.com/downloads/altair/software/8_inch_floppy/CPM/CPM%202.2/CPM%202.2B/)
folder on deramp.com: Mike Douglas's CP/M 2.2b for 56 KB of memory. Keep a
copy: the emulator writes what CP/M saves into the image.

**hello.bin.** Write the 32 bytes yourself:

```sh
echo 2110007eb7ca0e00d31123c30300760048454c4c4f2c20414c544149520d0a00 \
  | xxd -r -p > roms/hello.bin
```

The program walks the string at 0010H and sends each byte to port 11H
until it reaches the 00H at the end, then halts:

```text
0000H  21H 10H 00H  LXI H,0010H   HL points at the string
0003H  7EH          MOV A,M       next character
0004H  B7H          ORA A         00H ends the string
0005H  CAH 0EH 00H  JZ 000EH
0008H  D3H 11H      OUT 11H       print it on the 88-2SIO
000AH  23H          INX H
000BH  C3H 03H 00H  JMP 0003H
000EH  76H          HLT
0010H  "HELLO, ALTAIR", 0DH, 0AH, 00H
```
