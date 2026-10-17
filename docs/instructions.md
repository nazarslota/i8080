# Instruction set

The full 8080 instruction set, grouped the way the
[Intel 8080 Microcomputer Systems User's Manual](https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf)
groups it. The catalog shows what each instruction does and whether the
emulator supports it yet. Details follow for the instructions the emulator
runs.

When you add an instruction, update its status in the catalog and write its
section under [Details](#details).

**Status:** ✅ supported · 🟡 partly supported · ⬜ not yet

## Notation

| Symbol | Meaning                                                          |
|--------|------------------------------------------------------------------|
| `r`    | an 8-bit register: `B`, `C`, `D`, `E`, `H`, `L` or `A`           |
| `M`    | the memory byte at the address in `HL`                           |
| `rp`   | a register pair: `B` (BC), `D` (DE), `H` (HL) or `SP`            |
| `n`    | an 8-bit constant stored after the opcode                        |
| `nn`   | a 16-bit constant stored after the opcode, low byte first        |
| `cc`   | a condition: `NZ`, `Z`, `NC`, `C`, `PO`, `PE`, `P` or `M`        |
| Flags  | `S` sign, `Z` zero, `AC` auxiliary carry, `P` parity, `CY` carry |

Cycles count clock ticks. Where two numbers appear, the second applies when
an operand is `M`. For conditional calls and returns, the first number
applies when the condition holds.

## Catalog

### Data transfer

| Instruction | Bytes | Cycles | Flags | What it does                                 | Status |
|-------------|-------|--------|-------|----------------------------------------------|--------|
| `MOV r,r`   | 1     | 5 / 7  | none  | Copies a byte between registers and `M`      | ✅     |
| `MVI r,n`   | 2     | 7 / 10 | none  | Loads a constant byte into a register or `M` | ✅     |
| `LXI rp,nn` | 3     | 10     | none  | Loads a 16-bit constant into a register pair | ⬜     |
| `LDA nn`    | 3     | 13     | none  | Loads `A` from address `nn`                  | ⬜     |
| `STA nn`    | 3     | 13     | none  | Stores `A` at address `nn`                   | ⬜     |
| `LHLD nn`   | 3     | 16     | none  | Loads `L` from `nn` and `H` from `nn + 1`    | ⬜     |
| `SHLD nn`   | 3     | 16     | none  | Stores `L` at `nn` and `H` at `nn + 1`       | ⬜     |
| `LDAX rp`   | 1     | 7      | none  | Loads `A` from the address in `BC` or `DE`   | ⬜     |
| `STAX rp`   | 1     | 7      | none  | Stores `A` at the address in `BC` or `DE`    | ⬜     |
| `XCHG`      | 1     | 4      | none  | Swaps `HL` and `DE`                          | ⬜     |

### Arithmetic

| Instruction | Bytes | Cycles | Flags       | What it does                            | Status |
|-------------|-------|--------|-------------|-----------------------------------------|--------|
| `ADD r`     | 1     | 4 / 7  | S Z AC P CY | `A = A + r`                             | ✅     |
| `ADI n`     | 2     | 7      | S Z AC P CY | `A = A + n`                             | ⬜     |
| `ADC r`     | 1     | 4 / 7  | S Z AC P CY | `A = A + r + CY`                        | ✅     |
| `ACI n`     | 2     | 7      | S Z AC P CY | `A = A + n + CY`                        | ⬜     |
| `SUB r`     | 1     | 4 / 7  | S Z AC P CY | `A = A - r`                             | ✅     |
| `SUI n`     | 2     | 7      | S Z AC P CY | `A = A - n`                             | ⬜     |
| `SBB r`     | 1     | 4 / 7  | S Z AC P CY | `A = A - r - CY`                        | ✅     |
| `SBI n`     | 2     | 7      | S Z AC P CY | `A = A - n - CY`                        | ⬜     |
| `INR r`     | 1     | 5 / 10 | S Z AC P    | `r = r + 1`, carry unchanged            | ⬜     |
| `DCR r`     | 1     | 5 / 10 | S Z AC P    | `r = r - 1`, carry unchanged            | ⬜     |
| `INX rp`    | 1     | 5      | none        | `rp = rp + 1`                           | ⬜     |
| `DCX rp`    | 1     | 5      | none        | `rp = rp - 1`                           | ⬜     |
| `DAD rp`    | 1     | 10     | CY          | `HL = HL + rp`                          | ⬜     |
| `DAA`       | 1     | 4      | S Z AC P CY | Adjusts `A` to two decimal digits (BCD) | ⬜     |

### Logical

| Instruction | Bytes | Cycles | Flags       | What it does                       | Status |
|-------------|-------|--------|-------------|------------------------------------|--------|
| `ANA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A AND r`                      | ⬜     |
| `ANI n`     | 2     | 7      | S Z AC P CY | `A = A AND n`                      | ⬜     |
| `XRA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A XOR r`                      | ⬜     |
| `XRI n`     | 2     | 7      | S Z AC P CY | `A = A XOR n`                      | ⬜     |
| `ORA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A OR r`                       | ⬜     |
| `ORI n`     | 2     | 7      | S Z AC P CY | `A = A OR n`                       | ⬜     |
| `CMP r`     | 1     | 4 / 7  | S Z AC P CY | Sets flags from `A - r`, keeps `A` | ⬜     |
| `CPI n`     | 2     | 7      | S Z AC P CY | Sets flags from `A - n`, keeps `A` | ⬜     |
| `RLC`       | 1     | 4      | CY          | Rotates `A` left, bit 7 into `CY`  | ⬜     |
| `RRC`       | 1     | 4      | CY          | Rotates `A` right, bit 0 into `CY` | ⬜     |
| `RAL`       | 1     | 4      | CY          | Rotates `A` left through `CY`      | ⬜     |
| `RAR`       | 1     | 4      | CY          | Rotates `A` right through `CY`     | ⬜     |
| `CMA`       | 1     | 4      | none        | Inverts every bit of `A`           | ⬜     |
| `CMC`       | 1     | 4      | CY          | Inverts `CY`                       | ⬜     |
| `STC`       | 1     | 4      | CY          | Sets `CY` to 1                     | ⬜     |

### Branch

| Instruction | Bytes | Cycles  | Flags | What it does                               | Status |
|-------------|-------|---------|-------|--------------------------------------------|--------|
| `JMP nn`    | 3     | 10      | none  | Jumps to `nn`                              | ⬜     |
| `Jcc nn`    | 3     | 10      | none  | Jumps to `nn` if condition `cc` holds      | ⬜     |
| `CALL nn`   | 3     | 17      | none  | Pushes `PC` and jumps to `nn`              | ⬜     |
| `Ccc nn`    | 3     | 17 / 11 | none  | Calls `nn` if condition `cc` holds         | ⬜     |
| `RET`       | 1     | 10      | none  | Pops `PC`, returning from a call           | ⬜     |
| `Rcc`       | 1     | 11 / 5  | none  | Returns if condition `cc` holds            | ⬜     |
| `RST n`     | 1     | 11      | none  | Calls address `8 × n`, for `n` from 0 to 7 | ⬜     |
| `PCHL`      | 1     | 5       | none  | Jumps to the address in `HL`               | ⬜     |

### Stack, I/O and machine control

| Instruction | Bytes | Cycles | Flags              | What it does                                   | Status |
|-------------|-------|--------|--------------------|------------------------------------------------|--------|
| `PUSH rp`   | 1     | 11     | none               | Pushes `BC`, `DE`, `HL` or `PSW` (A and flags) | ⬜     |
| `POP rp`    | 1     | 10     | all, for `POP PSW` | Pops `BC`, `DE`, `HL` or `PSW`                 | ⬜     |
| `XTHL`      | 1     | 18     | none               | Swaps `HL` with the two bytes on top of stack  | ⬜     |
| `SPHL`      | 1     | 5      | none               | Copies `HL` into `SP`                          | ⬜     |
| `IN n`      | 2     | 10     | none               | Reads port `n` into `A`                        | ⬜     |
| `OUT n`     | 2     | 10     | none               | Writes `A` to port `n`                         | ⬜     |
| `EI`        | 1     | 4      | none               | Enables interrupts after the next instruction  | ⬜     |
| `DI`        | 1     | 4      | none               | Disables interrupts                            | ⬜     |
| `HLT`       | 1     | 7      | none               | Stops the CPU until an interrupt arrives       | ✅     |
| `NOP`       | 1     | 4      | none               | Does nothing                                   | ✅     |

## Details

### NOP: do nothing

`NOP` · opcode `00H` · 1 byte · 4 cycles · flags: none

`NOP` changes no register, flag or memory byte. The CPU spends 4 cycles
fetching it and moves `PC` to the next byte. Programmers used it to fill
space, reserve bytes for a later patch or wait a few microseconds.

Zeroed memory reads as `NOP`, so a CPU running through empty RAM executes
`NOP` after `NOP` until it reaches something else.

A program that leaves room for a later fix:

```asm
0100H   MVI  B,05H   ; B = 05H
0102H   NOP          ; two spare bytes: a later patch can put
0103H   NOP          ; MVI C,n here without moving the code below
0104H   MVI  A,01H   ; runs right after the NOPs
```

| After       | `PC`    | `A`   | `B`   |
|-------------|---------|-------|-------|
| `MVI B,05H` | `0102H` | `00H` | `05H` |
| `NOP`       | `0103H` | `00H` | `05H` |
| `NOP`       | `0104H` | `00H` | `05H` |
| `MVI A,01H` | `0106H` | `01H` | `05H` |

### MVI: load a constant byte

`MVI dst,n` · opcode `00 ddd 110` · 2 bytes · 7 cycles (10 with `M`) ·
flags: none

`MVI` copies the byte that follows the opcode into `dst`. With `M` as the
destination, the byte goes to memory at the address in `HL`. The CPU reads
the opcode and the constant in two memory reads, which is why `MVI` takes 7
cycles and `NOP` takes 4. Writing to `M` adds a memory write and 3 more
cycles.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `MVI B,n`   | `06H`  | `MVI H,n`   | `26H`  |
| `MVI C,n`   | `0EH`  | `MVI L,n`   | `2EH`  |
| `MVI D,n`   | `16H`  | `MVI M,n`   | `36H`  |
| `MVI E,n`   | `1EH`  | `MVI A,n`   | `3EH`  |

```asm
0100H   MVI  B,42H   ; B = 42H                       bytes: 06 42
0102H   MVI  A,0FFH  ; A = FFH                       bytes: 3E FF
0104H   MVI  H,20H   ; H = 20H                       bytes: 26 20
0106H   MVI  L,00H   ; L = 00H, so HL = 2000H        bytes: 2E 00
0108H   MVI  M,99H   ; memory[2000H] = 99H           bytes: 36 99
```

| After        | `PC`    | `A`   | `B`   | `HL`    | `memory[2000H]` |
|--------------|---------|-------|-------|---------|-----------------|
| `MVI B,42H`  | `0102H` | `00H` | `42H` | `0000H` | `00H`           |
| `MVI A,0FFH` | `0104H` | `FFH` | `42H` | `0000H` | `00H`           |
| `MVI H,20H`  | `0106H` | `FFH` | `42H` | `2000H` | `00H`           |
| `MVI L,00H`  | `0108H` | `FFH` | `42H` | `2000H` | `00H`           |
| `MVI M,99H`  | `010AH` | `FFH` | `42H` | `2000H` | `99H`           |

Intel assembler writes `0FFH` with a leading zero. Without it, the assembler
would read `FFH` as a name.

### MOV: copy a byte

`MOV dst,src` · opcode `01 ddd sss` · 1 byte · 5 cycles (7 with `M`) ·
flags: none

`MOV` copies `src` into `dst` and leaves `src` as it was. Either operand can
be `M`, the memory byte at the address in `HL`. A memory operand costs one
more bus access, which is why `MOV` takes 7 cycles with `M` and 5 without.

`MOV M,M` doesn't exist. Its opcode, `76H`, belongs to `HLT`.

| dst \ src | `B`   | `C`   | `D`   | `E`   | `H`   | `L`   | `M`   | `A`   |
|-----------|-------|-------|-------|-------|-------|-------|-------|-------|
| `B`       | `40H` | `41H` | `42H` | `43H` | `44H` | `45H` | `46H` | `47H` |
| `C`       | `48H` | `49H` | `4AH` | `4BH` | `4CH` | `4DH` | `4EH` | `4FH` |
| `D`       | `50H` | `51H` | `52H` | `53H` | `54H` | `55H` | `56H` | `57H` |
| `E`       | `58H` | `59H` | `5AH` | `5BH` | `5CH` | `5DH` | `5EH` | `5FH` |
| `H`       | `60H` | `61H` | `62H` | `63H` | `64H` | `65H` | `66H` | `67H` |
| `L`       | `68H` | `69H` | `6AH` | `6BH` | `6CH` | `6DH` | `6EH` | `6FH` |
| `M`       | `70H` | `71H` | `72H` | `73H` | `74H` | `75H` | `HLT` | `77H` |
| `A`       | `78H` | `79H` | `7AH` | `7BH` | `7CH` | `7DH` | `7EH` | `7FH` |

```asm
0100H   MVI  B,42H   ; B = 42H                    bytes: 06 42
0102H   MOV  A,B     ; A = B = 42H                bytes: 78
0103H   MVI  H,20H   ; H = 20H                    bytes: 26 20
0105H   MVI  L,00H   ; L = 00H, so HL = 2000H     bytes: 2E 00
0107H   MOV  M,A     ; memory[2000H] = A = 42H    bytes: 77
0108H   MOV  C,M     ; C = memory[2000H] = 42H    bytes: 4E
```

| After       | `PC`    | `A`   | `B`   | `C`   | `HL`    | `memory[2000H]` |
|-------------|---------|-------|-------|-------|---------|-----------------|
| `MVI B,42H` | `0102H` | `00H` | `42H` | `00H` | `0000H` | `00H`           |
| `MOV A,B`   | `0103H` | `42H` | `42H` | `00H` | `0000H` | `00H`           |
| `MVI H,20H` | `0105H` | `42H` | `42H` | `00H` | `2000H` | `00H`           |
| `MVI L,00H` | `0107H` | `42H` | `42H` | `00H` | `2000H` | `00H`           |
| `MOV M,A`   | `0108H` | `42H` | `42H` | `00H` | `2000H` | `42H`           |
| `MOV C,M`   | `0109H` | `42H` | `42H` | `42H` | `2000H` | `42H`           |

### HLT: stop the CPU

`HLT` · opcode `76H` · 1 byte · 7 cycles · flags: none

`HLT` stops the CPU. `PC` points past the `HLT`, and the CPU fetches nothing
more until an interrupt or a reset arrives. After an interrupt, the CPU runs
the interrupt handler and then continues with the instruction after `HLT`.
On the Altair, the HLTA light on the front panel shows the halted state.

Intel placed `HLT` in the slot where `MOV M,M` would sit, `01 110 110`.

```asm
0100H   MVI  A,07H   ; A = 07H                    bytes: 3E 07
0102H   HLT          ; stop                       bytes: 76
0103H   MVI  A,08H   ; never runs                 bytes: 3E 08
```

| After       | `PC`    | `A`   | Halted |
|-------------|---------|-------|--------|
| `MVI A,07H` | `0102H` | `07H` | no     |
| `HLT`       | `0103H` | `07H` | yes    |
| next step   | `0103H` | `07H` | yes    |

**Emulator status:** nothing wakes the CPU yet. Interrupts come later.

### ADD: add to the accumulator

`ADD src` · opcode `10 000 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`ADD` adds `src` to `A` and keeps the low 8 bits of the sum in `A`. With
`M` as the source, the CPU reads the byte at the address in `HL`.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `ADD B`     | `80H`  | `ADD H`     | `84H`  |
| `ADD C`     | `81H`  | `ADD L`     | `85H`  |
| `ADD D`     | `82H`  | `ADD M`     | `86H`  |
| `ADD E`     | `83H`  | `ADD A`     | `87H`  |

`ADD` sets all five flags from the result:

| Flag | Set to 1 when                                          |
|------|--------------------------------------------------------|
| `S`  | bit 7 of the result is 1                               |
| `Z`  | the result is `00H`                                    |
| `AC` | the low 4 bits of the operands sum to more than `0FH`  |
| `P`  | the result has an even number of 1 bits                |
| `CY` | the full sum doesn't fit in 8 bits                     |

The first `ADD` below repeats the example from the Intel manual. `ADD A`
doubles `A`.

```asm
0100H   MVI  A,6CH   ; A = 6CH                        bytes: 3E 6C
0102H   MVI  D,2EH   ; D = 2EH                        bytes: 16 2E
0104H   ADD  D       ; A = 6CH + 2EH = 9AH            bytes: 82
0105H   ADD  A       ; 9AH + 9AH = 134H: A = 34H      bytes: 87
```

| After       | `PC`    | `A`   | `D`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,6CH` | `0102H` | `6CH` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `MVI D,2EH` | `0104H` | `6CH` | `2EH` | 0   | 0   | 0    | 0   | 0    |
| `ADD D`     | `0105H` | `9AH` | `2EH` | 1   | 0   | 1    | 1   | 0    |
| `ADD A`     | `0106H` | `34H` | `2EH` | 0   | 0   | 1    | 0   | 1    |

In `ADD D`, `CH + EH = 1AH` carries out of the low 4 bits, so `AC` is 1.
`9AH` is `1001 1010`: bit 7 gives `S`, and four 1 bits give `P`. In `ADD A`,
the sum `134H` needs 9 bits, so `CY` is 1 and `A` keeps `34H`.

### ADC: add with carry

`ADC src` · opcode `10 001 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`ADC` adds `src` and the carry flag to `A`. It sets the flags the way `ADD`
does, with the carry counted in the sum.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `ADC B`     | `88H`  | `ADC H`     | `8CH`  |
| `ADC C`     | `89H`  | `ADC L`     | `8DH`  |
| `ADC D`     | `8AH`  | `ADC M`     | `8EH`  |
| `ADC E`     | `8BH`  | `ADC A`     | `8FH`  |

Programs use `ADC` to add numbers wider than 8 bits: `ADD` sums the low
bytes, and `ADC` sums the next bytes together with the carry from below.
This program adds `00FFH` and `0001H` and puts `0100H` in `HL`:

```asm
0100H   MVI  A,0FFH  ; A = low byte of 00FFH            bytes: 3E FF
0102H   MVI  B,01H   ; B = low byte of 0001H            bytes: 06 01
0104H   ADD  B       ; FFH + 01H = 100H: A = 00H        bytes: 80
0105H   MOV  L,A     ; L = 00H                          bytes: 6F
0106H   MVI  A,00H   ; A = high byte of 00FFH           bytes: 3E 00
0108H   MVI  B,00H   ; B = high byte of 0001H           bytes: 06 00
010AH   ADC  B       ; A = 00H + 00H + CY = 01H         bytes: 88
010BH   MOV  H,A     ; H = 01H, so HL = 0100H           bytes: 67
```

| After        | `PC`    | `A`   | `B`   | `HL`    | `S` | `Z` | `AC` | `P` | `CY` |
|--------------|---------|-------|-------|---------|-----|-----|------|-----|------|
| `MVI A,0FFH` | `0102H` | `FFH` | `00H` | `0000H` | 0   | 0   | 0    | 0   | 0    |
| `MVI B,01H`  | `0104H` | `FFH` | `01H` | `0000H` | 0   | 0   | 0    | 0   | 0    |
| `ADD B`      | `0105H` | `00H` | `01H` | `0000H` | 0   | 1   | 1    | 1   | 1    |
| `MOV L,A`    | `0106H` | `00H` | `01H` | `0000H` | 0   | 1   | 1    | 1   | 1    |
| `MVI A,00H`  | `0108H` | `00H` | `01H` | `0000H` | 0   | 1   | 1    | 1   | 1    |
| `MVI B,00H`  | `010AH` | `00H` | `00H` | `0000H` | 0   | 1   | 1    | 1   | 1    |
| `ADC B`      | `010BH` | `01H` | `00H` | `0000H` | 0   | 0   | 0    | 0   | 0    |
| `MOV H,A`    | `010CH` | `01H` | `00H` | `0100H` | 0   | 0   | 0    | 0   | 0    |

`MOV` and `MVI` leave the flags alone, so the carry from `ADD B` survives
until `ADC B` reads it.

### SUB: subtract from the accumulator

`SUB src` · opcode `10 010 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`SUB` subtracts `src` from `A` and keeps the low 8 bits of the difference
in `A`. `SUB A` clears `A`, and programs used it for that.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `SUB B`     | `90H`  | `SUB H`     | `94H`  |
| `SUB C`     | `91H`  | `SUB L`     | `95H`  |
| `SUB D`     | `92H`  | `SUB M`     | `96H`  |
| `SUB E`     | `93H`  | `SUB A`     | `97H`  |

The 8080 has no subtractor. It adds the two's complement instead:
`A - src = A + NOT src + 1`. The flags come from that addition:

- `CY` is the carry out of bit 7, inverted. `CY = 1` means `src` was larger
  than `A` and the subtraction borrowed.
- `AC` is the carry out of bit 3, as it is. `AC = 0` means the low 4 bits
  borrowed.
- `S`, `Z` and `P` follow the result, as in `ADD`.

The first `SUB` below repeats the example from the Intel manual.

```asm
0100H   MVI  A,3EH   ; A = 3EH                      bytes: 3E 3E
0102H   SUB  A       ; A = 3EH - 3EH = 00H          bytes: 97
0103H   MVI  B,01H   ; B = 01H                      bytes: 06 01
0105H   SUB  B       ; A = 00H - 01H = FFH          bytes: 90
```

| After       | `PC`    | `A`   | `B`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,3EH` | `0102H` | `3EH` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `SUB A`     | `0103H` | `00H` | `00H` | 0   | 1   | 1    | 1   | 0    |
| `MVI B,01H` | `0105H` | `00H` | `01H` | 0   | 1   | 1    | 1   | 0    |
| `SUB B`     | `0106H` | `FFH` | `01H` | 1   | 0   | 0    | 1   | 1    |

In `SUB A`, the CPU computes `3EH + C1H + 1 = 100H`. The carry out of bit 7
inverts to `CY = 0`: no borrow. In `SUB B`, `00H + FEH + 1 = FFH` gives no
carry out, so `CY = 1`.

### SBB: subtract with borrow

`SBB src` · opcode `10 011 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`SBB` subtracts `src` and the carry flag from `A`. It sets the flags the
way `SUB` does. Programs pair it with `SUB` the way they pair `ADC` with
`ADD`: `SUB` for the low bytes, `SBB` for the rest.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `SBB B`     | `98H`  | `SBB H`     | `9CH`  |
| `SBB C`     | `99H`  | `SBB L`     | `9DH`  |
| `SBB D`     | `9AH`  | `SBB M`     | `9EH`  |
| `SBB E`     | `9BH`  | `SBB A`     | `9FH`  |

The `SBB L` below repeats the example from the Intel manual. The `SUB L`
before it sets the carry.

```asm
0100H   MVI  L,01H   ; L = 01H                      bytes: 2E 01
0102H   SUB  L       ; A = 00H - 01H = FFH, CY = 1  bytes: 95
0103H   MVI  A,04H   ; A = 04H                      bytes: 3E 04
0105H   MVI  L,02H   ; L = 02H                      bytes: 2E 02
0107H   SBB  L       ; A = 04H - 02H - CY = 01H     bytes: 9D
```

| After       | `PC`    | `A`   | `L`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI L,01H` | `0102H` | `00H` | `01H` | 0   | 0   | 0    | 0   | 0    |
| `SUB L`     | `0103H` | `FFH` | `01H` | 1   | 0   | 0    | 1   | 1    |
| `MVI A,04H` | `0105H` | `04H` | `01H` | 1   | 0   | 0    | 1   | 1    |
| `MVI L,02H` | `0107H` | `04H` | `02H` | 1   | 0   | 0    | 1   | 1    |
| `SBB L`     | `0108H` | `01H` | `02H` | 0   | 0   | 1    | 0   | 0    |

In `SBB L`, the CPU computes `04H + FDH + 0 = 101H`. The carry into the sum
is `NOT CY`, so a set carry flag takes 1 away.
