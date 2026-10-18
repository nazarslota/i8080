# Glossary

The terms that the other pages use, from A to Z.

**88-2SIO.** The MITS serial card with two ports. The terminal and the
paper tape reader hang on its port A, at I/O ports `10H` and `11H`.

**88-DCDD.** The MITS floppy disk controller for up to 16 eight-inch
drives, at I/O ports `08H` to `0AH`.

**BDOS (Basic Disk Operating System).** The part of CP/M that programs call
with `CALL 0005H`, for files and the console.

**BIOS (Basic Input/Output System).** The part of CP/M written for one
machine. It reads the disk and the console through the machine's ports, so
the CCP and the BDOS run unchanged on any 8080 computer.

**Bootstrap loader.** The 28-byte program that an owner toggled in to read
a BASIC tape. It reads the checksum loader from the tape, and that loader
reads BASIC.

**Bus.** The wires between the processor and everything else. In code,
`MemoryBus` and `IoBus`.

**CCP (Console Command Processor).** The part of CP/M that prints `A>` and
runs your commands.

**CP/M.** The most popular operating system for 8080 and Z80 computers in the
late 1970s. It runs programs stored as `.COM` files.

**DBL (disk boot loader).** The MITS program in a PROM at FF00H that reads
the first sectors of the disk in drive 0 and jumps to them.

**HLE (high-level emulation).** Emulating what an operating system does,
instead of the hardware below it. The diagnostics test uses it for two CP/M
calls.

**Host.** Your real computer, the one running the emulator.

**I/O port.** A numbered channel, 0 to 255 on the 8080. The processor talks
to devices through ports with `IN` and `OUT`.

**Intel HEX.** A text format for programs: each line holds an address, the
bytes for it and a checksum. PROM programmers read it.

**PROM (programmable read-only memory).** A memory chip that you program
once, with a PROM programmer. The CPU reads it like RAM but can't write it,
and it keeps its bytes without power. An EPROM is a PROM that ultraviolet
light erases.

**Raw mode.** A terminal mode that passes each key to the program at once.
The normal mode collects a whole line, shows what you type and handles
Ctrl+C itself.

**RUBOUT.** The Teletype key that sent `7FH` to erase a character. Your
Backspace key sends the same byte.

**S-100.** The Altair's 100-pin expansion bus. Other makers copied it, and it
became a standard.

**Sense switches.** The front panel switches A8 to A15. A program reads them
with `IN 0FFH`.

**TPA (Transient Program Area).** The memory where CP/M runs a program, from
`0100H` up to CP/M itself.
