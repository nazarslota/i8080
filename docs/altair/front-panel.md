# The front panel

The Altair 8800 has no keyboard, no screen and no ROM. When you switch it on,
memory holds noise. You talk to the machine through its front panel: a row of
lights and a row of toggle switches. With them you stop the CPU, enter a
program byte by byte and start it.

![An Altair 8800 with its cover off, on top of an 8-inch floppy drive](https://upload.wikimedia.org/wikipedia/commons/0/01/Altair_8800_Computer.jpg)

*An Altair 8800 on top of its floppy drive. Photo by Michael Holley, public
domain, from
[Wikimedia Commons](https://commons.wikimedia.org/wiki/File:Altair_8800_Computer.jpg).*

The upper row of the panel holds the lights, the lower row the switches. The
boards stand upright in the slots of the backplane at the bottom of the case.
From left to right this machine has an 8K memory card, the two cards of the
MITS floppy controller, two 16K memory cards, the 88-2SIO serial card, a
PROM card and the 8080 CPU card.

This page explains the switches and the lights, and how `altair::FrontPanel`
emulates them.

## The switches

### Address and data: A0 to A15

Sixteen switches, A15 on the left and A0 on the right. A switch up is a 1
bit. What they mean depends on the command you press next:

| Used by                      | Switches  | Meaning                    |
|------------------------------|-----------|----------------------------|
| EXAMINE                      | A0 to A15 | An address                 |
| DEPOSIT, DEPOSIT NEXT        | A0 to A7  | A data byte                |
| A running program, `IN 0FFH` | A8 to A15 | The sense switches, a byte |

The panel groups the switches in threes, and MITS printed its listings in
octal to match: you set `076` as three groups, `000 111 110`.

### The command switches

Each command switch springs back after you press it. Most of them work only
while the CPU is stopped.

| Switch       | What it does                                                       |
|--------------|--------------------------------------------------------------------|
| STOP         | Stops the CPU                                                      |
| RUN          | Starts the CPU from the address that the lights show               |
| SINGLE STEP  | Executes one machine cycle and stops again                         |
| EXAMINE      | Jumps to the address on A0 to A15 and shows the byte there         |
| EXAMINE NEXT | Moves to the next address and shows its byte                       |
| DEPOSIT      | Writes the byte on A0 to A7 at the current address                 |
| DEPOSIT NEXT | Moves to the next address and writes the byte on A0 to A7 there    |
| RESET        | Sets the program counter to 0000H                                  |
| PROTECT      | Protects the memory page at the current address, on cards that can |
| UNPROTECT    | Removes that protection                                            |
| AUX          | Two spare switches, for cards that want them                       |

## The lights

| Lights                                   | What they show                                                |
|------------------------------------------|---------------------------------------------------------------|
| A15 to A0                                | The address on the bus: during EXAMINE, the current address   |
| D7 to D0                                 | The byte on the bus: during EXAMINE, the byte at that address |
| INTE                                     | Interrupts are on (`EI`)                                      |
| PROT                                     | The memory at this address is protected                       |
| WAIT                                     | The CPU is stopped                                            |
| HLDA                                     | A device has taken over the bus                               |
| MEMR, INP, M1, OUT, HLTA, STACK, WO, INT | The 8080 status byte of the current bus cycle                 |

The last row comes from the 8080 itself. At the start of each bus cycle the
CPU puts a status byte on the data lines, and the panel latches it: `MEMR`
for a memory read, `INP` and `OUT` for a port, `M1` for an instruction
fetch, `HLTA` after `HLT`, `STACK` for a stack access, `WO` for a write and
`INT` for an interrupt. The Intel manual describes the status byte.

## Entering a program

To enter the three-byte program `MVI A,41H` and `HLT` at 0000H:

1. Press STOP.
2. Set A0 to A15 to `0000` and press EXAMINE. The address lights show
   0000H.
3. Set A0 to A7 to `076` (`3EH`, `MVI A`) and press DEPOSIT.
4. Set `101` (`41H`) and press DEPOSIT NEXT. The address lights show 0001H.
5. Set `166` (`76H`, `HLT`) and press DEPOSIT NEXT.
6. Set A0 to A15 back to `0000`, press EXAMINE, then RUN.

Altair owners entered the 28-byte BASIC bootstrap loader this way, then set
the sense switches and pressed RUN. See
[Loading Basic with the 88-2SIO Board](https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf)
by Martin Eberhard for the full procedure.

## The sense switches

While a program runs, it can read A8 to A15 with `IN 0FFH`, A15 as bit 7.
Programs use them as settings. Altair BASIC and its loaders read them to
learn which card carries the tape and the terminal:

| Switches   | Read by              | Picks                        |
|------------|----------------------|------------------------------|
| A8 to A11  | The BASIC 4.x loader | The card that reads the tape |
| A12 to A15 | BASIC 4.x            | The card with the terminal   |

For the 88-2SIO, BASIC 4.x expects every switch down, `00H`. A value that
matches no card stops the loader with an `I` error.

## Inside the panel

The panel has no wire to the program counter. It controls the CPU through
two things: the READY line, which makes the CPU wait, and the data bus. To
change `PC`, it feeds the CPU an instruction:

| Switch       | What the panel does                                                     |
|--------------|-------------------------------------------------------------------------|
| EXAMINE      | Puts `JMP` (`303`) on the data bus, then A0 to A7, then A8 to A15       |
| EXAMINE NEXT | Puts `NOP` (`000`) on the data bus                                      |
| DEPOSIT      | Puts A0 to A7 on the data bus and pulses MWRITE, without the CPU        |
| DEPOSIT NEXT | Runs EXAMINE NEXT, then DEPOSIT                                         |
| SINGLE STEP  | Releases READY for one machine cycle                                    |

The CPU fetches the `JMP` as if it came from memory and jumps to the address
on the switches. A `NOP` moves `PC` one byte on. The CPU then waits on the
next fetch, with `PC` on the address lines, so the lights show that address
and the byte there, and DEPOSIT writes to it.

MITS describes these circuits in the
[Altair 8800 Theory of Operation](https://altairclone.com/downloads/manuals/Altair%208800%20Theory%20of%20Operation.pdf),
pages 5 and 6.

## In the emulator

`Altair::front_panel()` returns a `FrontPanel`, with one method per command:

| Switch       | `FrontPanel` method |
|--------------|---------------------|
| A0 to A15    | `set_switches(u16)` |
| EXAMINE      | `examine()`         |
| EXAMINE NEXT | `examine_next()`    |
| DEPOSIT      | `deposit()`         |
| DEPOSIT NEXT | `deposit_next()`    |
| RUN          | `run()`             |
| STOP         | `stop()`            |
| SINGLE STEP  | `single_step()`     |
| RESET        | `reset()`           |

`examine` and `examine_next` feed the CPU a `JMP` or a `NOP` and let it
execute, as the panel does. `reset` pulses the CPU's RESET pin, which also
wakes a CPU that executed `HLT`. `single_step` executes a whole instruction,
not one machine cycle.

The four EXAMINE and DEPOSIT methods return the byte that the data lights
show. `toggle_in(address, bytes)` presses EXAMINE, DEPOSIT and DEPOSIT NEXT
for you, the way you would enter a program by hand.

The emulator leaves out PROTECT, UNPROTECT and AUX, which no program we run
needs, and has no lights to show yet. A window with the panel is on the
roadmap.
