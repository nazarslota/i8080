# Altair BASIC

Altair BASIC loads from its original paper tape, as on the real machine.
[roms/README.md](../../roms/README.md) says where to get the tapes.

```sh
cargo run --release -- basic 8k-4.0 "roms/8K BASIC Ver 4-0.tap"
```

You name the BASIC first, then its tape:

| BASIC              | Name           | Tape                         |
|--------------------|----------------|------------------------------|
| 4K BASIC 4.0       | `4k-4.0`       | `4K BASIC Ver 4-0.tap`       |
| 8K BASIC 4.0       | `8k-4.0`       | `8K BASIC Ver 4-0.tap`       |
| Extended BASIC 4.0 | `extended-4.0` | `Extended BASIC Ver 4-0.tap` |
| Extended BASIC 4.1 | `extended-4.1` | `Extended BASIC Ver 4-1.tap` |

The emulator toggles in the MITS bootstrap loader for that BASIC, and the
loader reads the tape through the 88-2SIO serial card.
[boot.md](../boot.md) shows each step. Press Ctrl+] to quit.

## The questions at start

Keep Caps Lock on: BASIC wants capitals.

| Question                | Answer                                      |
|-------------------------|---------------------------------------------|
| `MEMORY SIZE?`          | Enter, for all the memory                   |
| `TERMINAL WIDTH?`       | Enter, for 72 columns (4K and 8K)           |
| `LINEPRINTER?`          | `O` (Extended): the emulator has no printer |
| `WANT SIN-COS-TAN-ATN?` | `Y` keeps the functions, `N` drops them     |

In 4K BASIC the last question splits into `SIN?`, `SQR?` and `RND?`.

## A first program

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

Backspace erases a character: it sends RUBOUT (`7FH`), the key that Altair
BASIC erases on.

## Sense switches

BASIC reads the front panel's sense switches to find the card with the
terminal. BASIC 4.x on the 88-2SIO wants them all down, and the emulator
sets them so. `--switches` sets another value in hex, such as
`--switches 0C` for BASIC 3.x. [front-panel.md](../altair/front-panel.md)
explains the switches.
