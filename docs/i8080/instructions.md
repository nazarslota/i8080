# Instruction set

The full 8080 instruction set, grouped the way the
[Intel 8080 Microcomputer Systems User's Manual](https://bitsavers.org/components/intel/MCS80/98-153B_Intel_8080_Microcomputer_Systems_Users_Manual_197509.pdf)
groups it. The catalog lists every instruction on one line, and the
details show its opcodes, its flags and a short program with the registers
after each step. The emulator runs them all.

After you change an instruction, update its catalog row, its section under
[Details](#details) and its link in [Contents](#contents).

## Contents

- [Notation](#notation)
- [Catalog](#catalog)
    - [Data transfer](#data-transfer)
    - [Arithmetic](#arithmetic)
    - [Logical](#logical)
    - [Branch](#branch)
    - [Stack, I/O and machine control](#stack-io-and-machine-control)
- [Details](#details)
    - Data transfer: [`MOV`](#mov-copy-a-byte) · [`MVI`](#mvi-load-a-constant-byte) · [`LXI`](#lxi-load-a-register-pair) · [`LDA`](#lda-load-a-from-an-address) · [`STA`](#sta-store-a-at-an-address) · [`LHLD`](#lhld-load-hl-from-an-address) · [
      `SHLD`](#shld-store-hl-at-an-address) · [`LDAX`](#ldax-load-a-through-bc-or-de) · [`STAX`](#stax-store-a-through-bc-or-de) · [`XCHG`](#xchg-swap-hl-and-de)
    - Arithmetic: [`ADD`](#add-add-to-the-accumulator) · [`ADI…CPI`](#adi-aci-sui-sbi-ani-xri-ori-cpi-arithmetic-and-logic-with-a-constant) · [`ADC`](#adc-add-with-carry) · [`SUB`](#sub-subtract-from-the-accumulator) · [
      `SBB`](#sbb-subtract-with-borrow) · [`INR`](#inr-increment) · [`DCR`](#dcr-decrement) · [`INX`](#inx-increment-a-register-pair) · [`DCX`](#dcx-decrement-a-register-pair) · [`DAD`](#dad-add-a-register-pair-to-hl) · [`DAA`](#daa-decimal-adjust-a)
    - Logical: [`ANA`](#ana-and-with-the-accumulator) · [`XRA`](#xra-exclusive-or-with-the-accumulator) · [`ORA`](#ora-or-with-the-accumulator) · [`CMP`](#cmp-compare-with-the-accumulator) · [`RLC…RAR`](#rlc-rrc-ral-rar-rotate-a) · [
      `CMA`](#cma-complement-a) · [`CMC`](#cmc-complement-carry) · [`STC`](#stc-set-carry)
    - Branch: [`JMP`](#jmp-jump) · [`Jcc`](#jcc-jump-if-a-condition-holds) · [`CALL`](#call-call-a-subroutine) · [`Ccc`](#ccc-call-if-a-condition-holds) · [`RET`](#ret-return-from-a-subroutine) · [`Rcc`](#rcc-return-if-a-condition-holds) · [
      `RST`](#rst-restart) · [`PCHL`](#pchl-jump-to-the-address-in-hl)
    - Stack, I/O and machine control: [`PUSH`](#push-push-onto-the-stack) · [`POP`](#pop-pop-from-the-stack) · [`XTHL`](#xthl-swap-hl-with-the-top-of-the-stack) · [`SPHL`](#sphl-copy-hl-into-sp) · [`IN`](#in-read-an-io-port) · [
      `OUT`](#out-write-an-io-port) · [`EI`](#ei-enable-interrupts) · [`DI`](#di-disable-interrupts) · [`HLT`](#hlt-stop-the-cpu) · [`NOP`](#nop-do-nothing)

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

| Instruction | Bytes | Cycles | Flags | What it does                                 |
|-------------|-------|--------|-------|----------------------------------------------|
| `MOV r,r`   | 1     | 5 / 7  | none  | Copies a byte between registers and `M`      |
| `MVI r,n`   | 2     | 7 / 10 | none  | Loads a constant byte into a register or `M` |
| `LXI rp,nn` | 3     | 10     | none  | Loads a 16-bit constant into a register pair |
| `LDA nn`    | 3     | 13     | none  | Loads `A` from address `nn`                  |
| `STA nn`    | 3     | 13     | none  | Stores `A` at address `nn`                   |
| `LHLD nn`   | 3     | 16     | none  | Loads `L` from `nn` and `H` from `nn + 1`    |
| `SHLD nn`   | 3     | 16     | none  | Stores `L` at `nn` and `H` at `nn + 1`       |
| `LDAX rp`   | 1     | 7      | none  | Loads `A` from the address in `BC` or `DE`   |
| `STAX rp`   | 1     | 7      | none  | Stores `A` at the address in `BC` or `DE`    |
| `XCHG`      | 1     | 4      | none  | Swaps `HL` and `DE`                          |

### Arithmetic

| Instruction | Bytes | Cycles | Flags       | What it does                            |
|-------------|-------|--------|-------------|-----------------------------------------|
| `ADD r`     | 1     | 4 / 7  | S Z AC P CY | `A = A + r`                             |
| `ADI n`     | 2     | 7      | S Z AC P CY | `A = A + n`                             |
| `ADC r`     | 1     | 4 / 7  | S Z AC P CY | `A = A + r + CY`                        |
| `ACI n`     | 2     | 7      | S Z AC P CY | `A = A + n + CY`                        |
| `SUB r`     | 1     | 4 / 7  | S Z AC P CY | `A = A - r`                             |
| `SUI n`     | 2     | 7      | S Z AC P CY | `A = A - n`                             |
| `SBB r`     | 1     | 4 / 7  | S Z AC P CY | `A = A - r - CY`                        |
| `SBI n`     | 2     | 7      | S Z AC P CY | `A = A - n - CY`                        |
| `INR r`     | 1     | 5 / 10 | S Z AC P    | `r = r + 1`, carry unchanged            |
| `DCR r`     | 1     | 5 / 10 | S Z AC P    | `r = r - 1`, carry unchanged            |
| `INX rp`    | 1     | 5      | none        | `rp = rp + 1`                           |
| `DCX rp`    | 1     | 5      | none        | `rp = rp - 1`                           |
| `DAD rp`    | 1     | 10     | CY          | `HL = HL + rp`                          |
| `DAA`       | 1     | 4      | S Z AC P CY | Adjusts `A` to two decimal digits (BCD) |

### Logical

| Instruction | Bytes | Cycles | Flags       | What it does                       |
|-------------|-------|--------|-------------|------------------------------------|
| `ANA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A AND r`                      |
| `ANI n`     | 2     | 7      | S Z AC P CY | `A = A AND n`                      |
| `XRA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A XOR r`                      |
| `XRI n`     | 2     | 7      | S Z AC P CY | `A = A XOR n`                      |
| `ORA r`     | 1     | 4 / 7  | S Z AC P CY | `A = A OR r`                       |
| `ORI n`     | 2     | 7      | S Z AC P CY | `A = A OR n`                       |
| `CMP r`     | 1     | 4 / 7  | S Z AC P CY | Sets flags from `A - r`, keeps `A` |
| `CPI n`     | 2     | 7      | S Z AC P CY | Sets flags from `A - n`, keeps `A` |
| `RLC`       | 1     | 4      | CY          | Rotates `A` left, bit 7 into `CY`  |
| `RRC`       | 1     | 4      | CY          | Rotates `A` right, bit 0 into `CY` |
| `RAL`       | 1     | 4      | CY          | Rotates `A` left through `CY`      |
| `RAR`       | 1     | 4      | CY          | Rotates `A` right through `CY`     |
| `CMA`       | 1     | 4      | none        | Inverts every bit of `A`           |
| `CMC`       | 1     | 4      | CY          | Inverts `CY`                       |
| `STC`       | 1     | 4      | CY          | Sets `CY` to 1                     |

### Branch

| Instruction | Bytes | Cycles  | Flags | What it does                               |
|-------------|-------|---------|-------|--------------------------------------------|
| `JMP nn`    | 3     | 10      | none  | Jumps to `nn`                              |
| `Jcc nn`    | 3     | 10      | none  | Jumps to `nn` if condition `cc` holds      |
| `CALL nn`   | 3     | 17      | none  | Pushes `PC` and jumps to `nn`              |
| `Ccc nn`    | 3     | 17 / 11 | none  | Calls `nn` if condition `cc` holds         |
| `RET`       | 1     | 10      | none  | Pops `PC`, returning from a call           |
| `Rcc`       | 1     | 11 / 5  | none  | Returns if condition `cc` holds            |
| `RST n`     | 1     | 11      | none  | Calls address `8 × n`, for `n` from 0 to 7 |
| `PCHL`      | 1     | 5       | none  | Jumps to the address in `HL`               |

### Stack, I/O and machine control

| Instruction | Bytes | Cycles | Flags              | What it does                                   |
|-------------|-------|--------|--------------------|------------------------------------------------|
| `PUSH rp`   | 1     | 11     | none               | Pushes `BC`, `DE`, `HL` or `PSW` (A and flags) |
| `POP rp`    | 1     | 10     | all, for `POP PSW` | Pops `BC`, `DE`, `HL` or `PSW`                 |
| `XTHL`      | 1     | 18     | none               | Swaps `HL` with the two bytes on top of stack  |
| `SPHL`      | 1     | 5      | none               | Copies `HL` into `SP`                          |
| `IN n`      | 2     | 10     | none               | Reads port `n` into `A`                        |
| `OUT n`     | 2     | 10     | none               | Writes `A` to port `n`                         |
| `EI`        | 1     | 4      | none               | Enables interrupts after the next instruction  |
| `DI`        | 1     | 4      | none               | Disables interrupts                            |
| `HLT`       | 1     | 7      | none               | Stops the CPU until an interrupt arrives       |
| `NOP`       | 1     | 4      | none               | Does nothing                                   |

## Details

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

### LXI: load a register pair

`LXI rp,nn` · opcode `00 rp0 001` · 3 bytes · 10 cycles · flags: none

`LXI` loads the 16-bit constant after the opcode into a register pair. The
constant sits in memory low byte first: `LXI H,2000H` assembles to
`21 00 20`. Programs use `LXI H` to point `HL` at data before working on it
through `M`, and `LXI SP` to set up the stack.

| Instruction | Opcode |
|-------------|--------|
| `LXI B,nn`  | `01H`  |
| `LXI D,nn`  | `11H`  |
| `LXI H,nn`  | `21H`  |
| `LXI SP,nn` | `31H`  |

The 2 bits `rp` pick the pair: `00` is `BC`, `01` `DE`, `10` `HL` and `11`
`SP`. Intel assembler names a pair by its first register, so `LXI B` loads
`BC`.

```asm
0100H   LXI  H,2000H   ; HL = 2000H                  bytes: 21 00 20
0103H   LXI  SP,0F000H ; SP = F000H                  bytes: 31 00 F0
0106H   MVI  M,42H     ; memory[2000H] = 42H         bytes: 36 42
```

| After           | `PC`    | `HL`    | `SP`    | `memory[2000H]` |
|-----------------|---------|---------|---------|-----------------|
| `LXI H,2000H`   | `0103H` | `2000H` | `0000H` | `00H`           |
| `LXI SP,0F000H` | `0106H` | `2000H` | `F000H` | `00H`           |
| `MVI M,42H`     | `0108H` | `2000H` | `F000H` | `42H`           |

### LDA: load A from an address

`LDA nn` · opcode `3AH` · 3 bytes · 13 cycles · flags: none

`LDA` loads `A` from address `nn`.

The example repeats the one from the Intel manual. Memory at `0300H` holds
`77H` before the program runs.

```asm
0100H   LDA  0300H    ; A = memory[0300H] = 77H      bytes: 3A 00 03
```

| After       | `PC`    | `A`   |
|-------------|---------|-------|
| `LDA 0300H` | `0103H` | `77H` |

### STA: store A at an address

`STA nn` · opcode `32H` · 3 bytes · 13 cycles · flags: none

`STA` stores `A` at address `nn`. Unlike `MOV M,A`, it needs no pair set up
first, and it costs 13 cycles instead of 7.

The example repeats the one from the Intel manual.

```asm
0100H   MVI  A,42H    ; A = 42H                      bytes: 3E 42
0102H   STA  05B3H    ; memory[05B3H] = A = 42H      bytes: 32 B3 05
```

| After       | `PC`    | `A`   | `memory[05B3H]` |
|-------------|---------|-------|-----------------|
| `MVI A,42H` | `0102H` | `42H` | `00H`           |
| `STA 05B3H` | `0105H` | `42H` | `42H`           |

### LHLD: load HL from an address

`LHLD nn` · opcode `2AH` · 3 bytes · 16 cycles · flags: none

`LHLD` loads `L` from address `nn` and `H` from `nn + 1`. Programs keep
their 16-bit variables in memory and pair `LHLD` with `SHLD` to read and
write them.

The example repeats the one from the Intel manual. Memory at `025BH` holds
`FFH` and at `025CH` holds `03H` before the program runs.

```asm
0100H   LHLD 025BH    ; L = FFH, H = 03H             bytes: 2A 5B 02
```

| After        | `PC`    | `H`   | `L`   |
|--------------|---------|-------|-------|
| `LHLD 025BH` | `0103H` | `03H` | `FFH` |

### SHLD: store HL at an address

`SHLD nn` · opcode `22H` · 3 bytes · 16 cycles · flags: none

`SHLD` stores `L` at address `nn` and `H` at `nn + 1`. The low byte goes
first, the same order the 8080 uses for 16-bit constants in code.

The example repeats the one from the Intel manual.

```asm
0100H   LXI  H,0AE29H ; HL = AE29H                   bytes: 21 29 AE
0103H   SHLD 010AH    ; memory[010AH] = L = 29H,     bytes: 22 0A 01
                      ; memory[010BH] = H = AEH
```

| After          | `PC`    | `HL`    | `memory[010AH]` | `memory[010BH]` |
|----------------|---------|---------|-----------------|-----------------|
| `LXI H,0AE29H` | `0103H` | `AE29H` | `00H`           | `00H`           |
| `SHLD 010AH`   | `0106H` | `AE29H` | `29H`           | `AEH`           |

### LDAX: load A through BC or DE

`LDAX rp` · opcode `00 rp1 010` · 1 byte · 7 cycles · flags: none

`LDAX` loads `A` from the address in `BC` or `DE`. It works like `MOV A,M`,
with another pair in place of `HL`. A copy loop can read through `DE` with
`LDAX D` and write through `HL` with `MOV M,A`.

| Instruction | Opcode |
|-------------|--------|
| `LDAX B`    | `0AH`  |
| `LDAX D`    | `1AH`  |

The example repeats the one from the Intel manual. Memory at `938BH` holds
`77H` before the program runs.

```asm
0100H   LXI  D,938BH  ; DE = 938BH                   bytes: 11 8B 93
0103H   LDAX D        ; A = memory[938BH] = 77H      bytes: 1A
```

| After         | `PC`    | `A`   | `DE`    |
|---------------|---------|-------|---------|
| `LXI D,938BH` | `0103H` | `00H` | `938BH` |
| `LDAX D`      | `0104H` | `77H` | `938BH` |

### STAX: store A through BC or DE

`STAX rp` · opcode `00 rp0 010` · 1 byte · 7 cycles · flags: none

`STAX` stores `A` at the address in `BC` or `DE`. It works like `MOV M,A`,
with another pair in place of `HL`.

| Instruction | Opcode |
|-------------|--------|
| `STAX B`    | `02H`  |
| `STAX D`    | `12H`  |

`STAX H` and `STAX SP` don't exist: `MOV M,A` covers `HL`, and their slots
hold `SHLD` and `STA`.

The example repeats the one from the Intel manual.

```asm
0100H   MVI  A,42H    ; A = 42H                      bytes: 3E 42
0102H   LXI  B,3F16H  ; BC = 3F16H                   bytes: 01 16 3F
0105H   STAX B        ; memory[3F16H] = A = 42H      bytes: 02
```

| After         | `PC`    | `A`   | `BC`    | `memory[3F16H]` |
|---------------|---------|-------|---------|-----------------|
| `MVI A,42H`   | `0102H` | `42H` | `0000H` | `00H`           |
| `LXI B,3F16H` | `0105H` | `42H` | `3F16H` | `00H`           |
| `STAX B`      | `0106H` | `42H` | `3F16H` | `42H`           |

### XCHG: swap HL and DE

`XCHG` · opcode `EBH` · 1 byte · 4 cycles · flags: none

`XCHG` swaps `H` with `D` and `L` with `E`. Only `HL` works with `M`, `DAD`
and `SHLD`, so programs swap a pointer into `HL`, use it there and swap it
back.

The example repeats the one from the Intel manual.

```asm
0100H   LXI  D,3355H  ; DE = 3355H                   bytes: 11 55 33
0103H   LXI  H,00FFH  ; HL = 00FFH                   bytes: 21 FF 00
0106H   XCHG          ; DE = 00FFH, HL = 3355H       bytes: EB
```

| After         | `PC`    | `DE`    | `HL`    |
|---------------|---------|---------|---------|
| `LXI D,3355H` | `0103H` | `3355H` | `0000H` |
| `LXI H,00FFH` | `0106H` | `3355H` | `00FFH` |
| `XCHG`        | `0107H` | `00FFH` | `3355H` |

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

| Flag | Set to 1 when                                         |
|------|-------------------------------------------------------|
| `S`  | bit 7 of the result is 1                              |
| `Z`  | the result is `00H`                                   |
| `AC` | the low 4 bits of the operands sum to more than `0FH` |
| `P`  | the result has an even number of 1 bits               |
| `CY` | the full sum doesn't fit in 8 bits                    |

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

### ADI, ACI, SUI, SBI, ANI, XRI, ORI, CPI: arithmetic and logic with a constant

`ADI n` and the rest · opcode `11 yyy 110` · 2 bytes · 7 cycles ·
flags: S Z AC P CY

Each of these does what its register twin does, with the byte after the
opcode in place of `src`. `ADI 03H` adds 3 to `A` the way `ADD B` adds `B`,
and the flags follow the same rules. The CPU reads the opcode and the
constant in two memory reads, so each takes 7 cycles, like `MVI`.

| Instruction | Opcode | What it does                       | Register twin                                   |
|-------------|--------|------------------------------------|-------------------------------------------------|
| `ADI n`     | `C6H`  | `A = A + n`                        | [`ADD`](#add-add-to-the-accumulator)            |
| `ACI n`     | `CEH`  | `A = A + n + CY`                   | [`ADC`](#adc-add-with-carry)                    |
| `SUI n`     | `D6H`  | `A = A - n`                        | [`SUB`](#sub-subtract-from-the-accumulator)     |
| `SBI n`     | `DEH`  | `A = A - n - CY`                   | [`SBB`](#sbb-subtract-with-borrow)              |
| `ANI n`     | `E6H`  | `A = A AND n`                      | [`ANA`](#ana-and-with-the-accumulator)          |
| `XRI n`     | `EEH`  | `A = A XOR n`                      | [`XRA`](#xra-exclusive-or-with-the-accumulator) |
| `ORI n`     | `F6H`  | `A = A OR n`                       | [`ORA`](#ora-or-with-the-accumulator)           |
| `CPI n`     | `FEH`  | Sets flags from `A - n`, keeps `A` | [`CMP`](#cmp-compare-with-the-accumulator)      |

The 3 bits `yyy` pick the operation in the same order as in `10 yyy sss`:
`ADD` is `000`, `CMP` is `111`. That is why the opcodes go up in steps of 8.

Three of them need a closer look:

- `ACI` and `SBI` read `CY` left by the instruction before them. Chained
  after `ADI` or `SUI`, they add and subtract numbers wider than 8 bits.
- `CPI` subtracts and throws the difference away. `A` keeps its value, and
  the flags read the same way as after `CMP`: `Z = 1` means `A` equals `n`,
  `CY = 1` means `A` is less than `n`.
- `ANI` sets `AC` to bit 3 of `A OR n`, like `ANA`. Some Intel manuals say
  `ANI` clears `AC`; the 8080 does not, and 8080EXM checks for that.

The program below runs all eight. Watch `CY` pass from `SUI` to `ACI` and
from `ACI` to `SBI`.

```asm
0100H   MVI  A,0AH   ; A = 0AH                           bytes: 3E 0A
0102H   ADI  03H     ; A = 0AH + 03H = 0DH               bytes: C6 03
0104H   SUI  0EH     ; A = 0DH - 0EH = FFH, borrow       bytes: D6 0E
0106H   ACI  01H     ; A = FFH + 01H + CY = 101H: 01H    bytes: CE 01
0108H   SBI  00H     ; A = 01H - 00H - CY = 00H          bytes: DE 00
010AH   ORI  0F0H    ; A = 00H OR F0H = F0H              bytes: F6 F0
010CH   ANI  3CH     ; A = F0H AND 3CH = 30H             bytes: E6 3C
010EH   XRI  33H     ; A = 30H XOR 33H = 03H             bytes: EE 33
0110H   CPI  03H     ; A stays 03H, Z = 1: equal         bytes: FE 03
```

| After       | `PC`    | `A`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-----|-----|------|-----|------|
| `MVI A,0AH` | `0102H` | `0AH` | 0   | 0   | 0    | 0   | 0    |
| `ADI 03H`   | `0104H` | `0DH` | 0   | 0   | 0    | 0   | 0    |
| `SUI 0EH`   | `0106H` | `FFH` | 1   | 0   | 0    | 1   | 1    |
| `ACI 01H`   | `0108H` | `01H` | 0   | 0   | 1    | 0   | 1    |
| `SBI 00H`   | `010AH` | `00H` | 0   | 1   | 1    | 1   | 0    |
| `ORI 0F0H`  | `010CH` | `F0H` | 1   | 0   | 0    | 1   | 0    |
| `ANI 3CH`   | `010EH` | `30H` | 0   | 0   | 1    | 1   | 0    |
| `XRI 33H`   | `0110H` | `03H` | 0   | 0   | 0    | 1   | 0    |
| `CPI 03H`   | `0112H` | `03H` | 0   | 1   | 1    | 1   | 0    |

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

### INR: increment

`INR dst` · opcode `00 ddd 100` · 1 byte · 5 cycles (10 with `M`) ·
flags: S Z AC P

`INR` adds 1 to a register or to `M`, the memory byte at the address in
`HL`. `FFH` wraps to `00H`. With `M`, the CPU reads the byte, adds 1 and
writes it back, so it takes 5 more cycles.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `INR B`     | `04H`  | `INR H`     | `24H`  |
| `INR C`     | `0CH`  | `INR L`     | `2CH`  |
| `INR D`     | `14H`  | `INR M`     | `34H`  |
| `INR E`     | `1CH`  | `INR A`     | `3CH`  |

`INR` sets `S`, `Z`, `AC` and `P` the way `ADD` with 1 would, and leaves
`CY` as it was. A loop can count in one register while `CY` carries a
multi-byte sum from one pass to the next.

The `INR C` below repeats the example from the Intel manual. `INR A` wraps
`FFH` to `00H` and still leaves `CY` at 0.

```asm
0100H   MVI  C,99H   ; C = 99H                       bytes: 0E 99
0102H   INR  C       ; C = 99H + 1 = 9AH             bytes: 0C
0103H   MVI  A,0FFH  ; A = FFH                       bytes: 3E FF
0105H   INR  A       ; A = 00H, CY stays 0           bytes: 3C
```

| After        | `PC`    | `A`   | `C`   | `S` | `Z` | `AC` | `P` | `CY` |
|--------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI C,99H`  | `0102H` | `00H` | `99H` | 0   | 0   | 0    | 0   | 0    |
| `INR C`      | `0103H` | `00H` | `9AH` | 1   | 0   | 0    | 1   | 0    |
| `MVI A,0FFH` | `0105H` | `FFH` | `9AH` | 1   | 0   | 0    | 1   | 0    |
| `INR A`      | `0106H` | `00H` | `9AH` | 0   | 1   | 1    | 1   | 0    |

### DCR: decrement

`DCR dst` · opcode `00 ddd 101` · 1 byte · 5 cycles (10 with `M`) ·
flags: S Z AC P

`DCR` subtracts 1 from a register or from `M`. `00H` wraps to `FFH`. It sets
`S`, `Z`, `AC` and `P` the way `SUB` with 1 would, and leaves `CY` as it
was. `AC = 0` means the low 4 bits borrowed, as after `SUB`: `10H` drops to
`0FH` with `AC = 0`.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `DCR B`     | `05H`  | `DCR H`     | `25H`  |
| `DCR C`     | `0DH`  | `DCR L`     | `2DH`  |
| `DCR D`     | `15H`  | `DCR M`     | `35H`  |
| `DCR E`     | `1DH`  | `DCR A`     | `3DH`  |

Programs count loops down with `DCR`: load a counter, run the loop body,
`DCR` the counter and jump back while `Z = 0`. The jump comes with `JNZ`
later.

The `DCR M` below repeats the example from the Intel manual.

```asm
0100H   MVI  H,3AH   ; H = 3AH                       bytes: 26 3A
0102H   MVI  L,7CH   ; L = 7CH, so HL = 3A7CH        bytes: 2E 7C
0104H   MVI  M,40H   ; memory[3A7CH] = 40H           bytes: 36 40
0106H   DCR  M       ; memory[3A7CH] = 40H - 1 = 3FH bytes: 35
0107H   DCR  A       ; A = 00H - 1 = FFH, CY stays 0 bytes: 3D
```

| After       | `PC`    | `A`   | `HL`    | `memory[3A7CH]` | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|---------|-----------------|-----|-----|------|-----|------|
| `MVI H,3AH` | `0102H` | `00H` | `3A00H` | `00H`           | 0   | 0   | 0    | 0   | 0    |
| `MVI L,7CH` | `0104H` | `00H` | `3A7CH` | `00H`           | 0   | 0   | 0    | 0   | 0    |
| `MVI M,40H` | `0106H` | `00H` | `3A7CH` | `40H`           | 0   | 0   | 0    | 0   | 0    |
| `DCR M`     | `0107H` | `00H` | `3A7CH` | `3FH`           | 0   | 0   | 0    | 1   | 0    |
| `DCR A`     | `0108H` | `FFH` | `3A7CH` | `3FH`           | 1   | 0   | 0    | 1   | 0    |

### INX: increment a register pair

`INX rp` · opcode `00 rp0 011` · 1 byte · 5 cycles · flags: none

`INX` adds 1 to a register pair. `FFFFH` wraps to `0000H`. Programs step
`HL` through a block of memory with `INX H`.

| Instruction | Opcode |
|-------------|--------|
| `INX B`     | `03H`  |
| `INX D`     | `13H`  |
| `INX H`     | `23H`  |
| `INX SP`    | `33H`  |

`INX` sets no flags, even when the pair wraps to `0000H`. A program can't
test `Z` after it.

The example repeats the one from the Intel manual: the carry from `E`
moves into `D`.

```asm
0100H   LXI  D,38FFH  ; DE = 38FFH                   bytes: 11 FF 38
0103H   INX  D        ; DE = 38FFH + 1 = 3900H       bytes: 13
```

| After         | `PC`    | `D`   | `E`   |
|---------------|---------|-------|-------|
| `LXI D,38FFH` | `0103H` | `38H` | `FFH` |
| `INX D`       | `0104H` | `39H` | `00H` |

### DCX: decrement a register pair

`DCX rp` · opcode `00 rp1 011` · 1 byte · 5 cycles · flags: none

`DCX` subtracts 1 from a register pair. `0000H` wraps to `FFFFH`.

| Instruction | Opcode |
|-------------|--------|
| `DCX B`     | `0BH`  |
| `DCX D`     | `1BH`  |
| `DCX H`     | `2BH`  |
| `DCX SP`    | `3BH`  |

Like `INX`, `DCX` sets no flags. A loop that counts a 16-bit pair down to 0
can't jump on `Z` after `DCX B`. It has to test the two halves itself,
for example with `MOV A,B` and `ORA C`.

The first `DCX H` repeats the example from the Intel manual.

```asm
0100H   LXI  H,9800H  ; HL = 9800H                   bytes: 21 00 98
0103H   DCX  H        ; HL = 9800H - 1 = 97FFH       bytes: 2B
0104H   LXI  B,0001H  ; BC = 0001H                   bytes: 01 01 00
0107H   DCX  B        ; BC = 0000H, Z stays 0        bytes: 0B
```

| After         | `PC`    | `BC`    | `HL`    | `Z` |
|---------------|---------|---------|---------|-----|
| `LXI H,9800H` | `0103H` | `0000H` | `9800H` | 0   |
| `DCX H`       | `0104H` | `0000H` | `97FFH` | 0   |
| `LXI B,0001H` | `0107H` | `0001H` | `97FFH` | 0   |
| `DCX B`       | `0108H` | `0000H` | `97FFH` | 0   |

### DAD: add a register pair to HL

`DAD rp` · opcode `00 rp1 001` · 1 byte · 10 cycles · flags: CY

`DAD` adds a register pair to `HL` and keeps the low 16 bits of the sum in
`HL`. It sets `CY` when the sum needs 17 bits and leaves the other flags
alone. `DAD H` doubles `HL`, which shifts it left by one bit, so programs
use it to multiply.

| Instruction | Opcode |
|-------------|--------|
| `DAD B`     | `09H`  |
| `DAD D`     | `19H`  |
| `DAD H`     | `29H`  |
| `DAD SP`    | `39H`  |

The `DAD B` below repeats the example from the Intel manual.

```asm
0100H   LXI  B,339FH  ; BC = 339FH                   bytes: 01 9F 33
0103H   LXI  H,0A17BH ; HL = A17BH                   bytes: 21 7B A1
0106H   DAD  B        ; HL = A17BH + 339FH = D51AH   bytes: 09
0107H   DAD  H        ; D51AH + D51AH = 1AA34H       bytes: 29
```

| After          | `PC`    | `BC`    | `HL`    | `CY` |
|----------------|---------|---------|---------|------|
| `LXI B,339FH`  | `0103H` | `339FH` | `0000H` | 0    |
| `LXI H,0A17BH` | `0106H` | `339FH` | `A17BH` | 0    |
| `DAD B`        | `0107H` | `339FH` | `D51AH` | 0    |
| `DAD H`        | `0108H` | `339FH` | `AA34H` | 1    |

In `DAD H`, the sum `1AA34H` needs 17 bits. `HL` keeps `AA34H` and `CY`
takes the 17th bit.

### DAA: decimal adjust A

`DAA` · opcode `27H` · 1 byte · 4 cycles · flags: S Z AC P CY

`DAA` turns the binary sum of two BCD bytes into a BCD byte. In BCD (binary-coded decimal) each 4-bit half of a byte holds one decimal digit
from 0 to 9: `42H` means 42. Programs that show numbers on 7-segment
displays, such as calculators and clocks, kept them in BCD.

`ADD` knows nothing about BCD. `15H + 27H` gives `3CH`, and `C` isn't a
decimal digit. `DAA` fixes the sum in two steps:

1. If the low digit is above 9, or `AC = 1` because it overflowed, `DAA`
   adds `06H`. Adding 6 skips the six values `AH` to `FH`.
2. If the high digit is above 9, or `CY = 1`, `DAA` adds `60H` and sets
   `CY` to 1. `CY = 1` then means the sum passed 99.

`DAA` sets `S`, `Z`, `AC` and `P` from the result of that correction. On
the 8080 it works only after an addition: the CPU keeps no flag that
remembers a subtraction. The Z80 added one for that.

The last `DAA` below repeats the example from the Intel manual.

```asm
0100H   MVI  A,15H   ; A = 15H, BCD for 15            bytes: 3E 15
0102H   ADI  27H     ; A = 15H + 27H = 3CH            bytes: C6 27
0104H   DAA          ; A = 3CH + 06H = 42H, BCD 42    bytes: 27
0106H   ADI  59H     ; A = 42H + 59H = 9BH            bytes: C6 59
0108H   DAA          ; A = 9BH + 66H = 01H, CY = 1    bytes: 27
```

| After       | `PC`    | `A`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-----|-----|------|-----|------|
| `MVI A,15H` | `0102H` | `15H` | 0   | 0   | 0    | 0   | 0    |
| `ADI 27H`   | `0104H` | `3CH` | 0   | 0   | 0    | 1   | 0    |
| `DAA`       | `0105H` | `42H` | 0   | 0   | 1    | 1   | 0    |
| `ADI 59H`   | `0107H` | `9BH` | 1   | 0   | 0    | 0   | 0    |
| `DAA`       | `0108H` | `01H` | 0   | 0   | 1    | 0   | 1    |

The second `DAA` adds `06H` because the low digit `B` is above 9, and `60H`
because the high digit becomes `A` after that. `42 + 59 = 101`: `A` holds
the last two digits, `01`, and `CY` holds the hundred.

### ANA: AND with the accumulator

`ANA src` · opcode `10 100 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`ANA` keeps the bits of `A` where `src` has a 1 and clears the rest. Programs
use it to pick bits out of a byte: `ANI 0FH` and `ANA` with `0FH` in a
register keep the low 4 bits.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `ANA B`     | `A0H`  | `ANA H`     | `A4H`  |
| `ANA C`     | `A1H`  | `ANA L`     | `A5H`  |
| `ANA D`     | `A2H`  | `ANA M`     | `A6H`  |
| `ANA E`     | `A3H`  | `ANA A`     | `A7H`  |

`ANA` sets `CY` to 0 and takes `S`, `Z` and `P` from the result. The 8080
sets `AC` to bit 3 of `A OR src`. The Intel 8085 sets `AC` to 1 instead, and
CPU diagnostics such as 8080EXM check the 8080 behavior.

The example repeats the one from the Intel manual.

```asm
0100H   MVI  A,0FCH  ; A = FCH = 1111 1100           bytes: 3E FC
0102H   MVI  C,0FH   ; C = 0FH = 0000 1111           bytes: 0E 0F
0104H   ANA  C       ; A = FCH AND 0FH = 0CH         bytes: A1
```

| After        | `PC`    | `A`   | `C`   | `S` | `Z` | `AC` | `P` | `CY` |
|--------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,0FCH` | `0102H` | `FCH` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `MVI C,0FH`  | `0104H` | `FCH` | `0FH` | 0   | 0   | 0    | 0   | 0    |
| `ANA C`      | `0105H` | `0CH` | `0FH` | 0   | 0   | 1    | 1   | 0    |

### XRA: exclusive OR with the accumulator

`XRA src` · opcode `10 101 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`XRA` flips the bits of `A` where `src` has a 1. `XRA` with `FFH` inverts
every bit, and `XRA A` sets `A` to 0. Programs used `XRA A` to clear `A` and
`CY` in one byte.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `XRA B`     | `A8H`  | `XRA H`     | `ACH`  |
| `XRA C`     | `A9H`  | `XRA L`     | `ADH`  |
| `XRA D`     | `AAH`  | `XRA M`     | `AEH`  |
| `XRA E`     | `ABH`  | `XRA A`     | `AFH`  |

`XRA` sets `AC` and `CY` to 0 and takes `S`, `Z` and `P` from the result.

```asm
0100H   MVI  A,5CH   ; A = 5CH = 0101 1100           bytes: 3E 5C
0102H   MVI  B,0FFH  ; B = FFH                       bytes: 06 FF
0104H   XRA  B       ; A = 5CH XOR FFH = A3H         bytes: A8
0105H   XRA  A       ; A = A3H XOR A3H = 00H         bytes: AF
```

| After        | `PC`    | `A`   | `B`   | `S` | `Z` | `AC` | `P` | `CY` |
|--------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,5CH`  | `0102H` | `5CH` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `MVI B,0FFH` | `0104H` | `5CH` | `FFH` | 0   | 0   | 0    | 0   | 0    |
| `XRA B`      | `0105H` | `A3H` | `FFH` | 1   | 0   | 0    | 1   | 0    |
| `XRA A`      | `0106H` | `00H` | `FFH` | 0   | 1   | 0    | 1   | 0    |

### ORA: OR with the accumulator

`ORA src` · opcode `10 110 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`ORA` sets the bits of `A` where `src` has a 1 and keeps the rest. `ORA A`
leaves `A` as it is and sets the flags from it, so programs use it to test
`A` for 0.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `ORA B`     | `B0H`  | `ORA H`     | `B4H`  |
| `ORA C`     | `B1H`  | `ORA L`     | `B5H`  |
| `ORA D`     | `B2H`  | `ORA M`     | `B6H`  |
| `ORA E`     | `B3H`  | `ORA A`     | `B7H`  |

`ORA` sets `AC` and `CY` to 0 and takes `S`, `Z` and `P` from the result.

The first `ORA` repeats the example from the Intel manual.

```asm
0100H   MVI  A,33H   ; A = 33H = 0011 0011           bytes: 3E 33
0102H   MVI  C,0FH   ; C = 0FH = 0000 1111           bytes: 0E 0F
0104H   ORA  C       ; A = 33H OR 0FH = 3FH          bytes: B1
0105H   MVI  A,00H   ; A = 00H                       bytes: 3E 00
0107H   ORA  A       ; A stays 00H, Z = 1            bytes: B7
```

| After       | `PC`    | `A`   | `C`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,33H` | `0102H` | `33H` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `MVI C,0FH` | `0104H` | `33H` | `0FH` | 0   | 0   | 0    | 0   | 0    |
| `ORA C`     | `0105H` | `3FH` | `0FH` | 0   | 0   | 0    | 1   | 0    |
| `MVI A,00H` | `0107H` | `00H` | `0FH` | 0   | 0   | 0    | 1   | 0    |
| `ORA A`     | `0108H` | `00H` | `0FH` | 0   | 1   | 0    | 1   | 0    |

### CMP: compare with the accumulator

`CMP src` · opcode `10 111 sss` · 1 byte · 4 cycles (7 with `M`) ·
flags: S Z AC P CY

`CMP` subtracts `src` from `A` the way `SUB` does, sets the flags and throws
the difference away. `A` keeps its value. A conditional jump after `CMP`
reads the answer:

| After `CMP src` | Means                     |
|-----------------|---------------------------|
| `Z = 1`         | `A` equals `src`          |
| `CY = 1`        | `A` is less than `src`    |
| `Z = 0, CY = 0` | `A` is greater than `src` |

`CMP` compares unsigned bytes: `E5H` counts as larger than `05H`.

| Instruction | Opcode | Instruction | Opcode |
|-------------|--------|-------------|--------|
| `CMP B`     | `B8H`  | `CMP H`     | `BCH`  |
| `CMP C`     | `B9H`  | `CMP L`     | `BDH`  |
| `CMP D`     | `BAH`  | `CMP M`     | `BEH`  |
| `CMP E`     | `BBH`  | `CMP A`     | `BFH`  |

Both `CMP E` below repeat the examples from the Intel manual.

```asm
0100H   MVI  A,0AH   ; A = 0AH                       bytes: 3E 0A
0102H   MVI  E,05H   ; E = 05H                       bytes: 1E 05
0104H   CMP  E       ; 0AH - 05H: A is larger        bytes: BB
0105H   MVI  A,02H   ; A = 02H                       bytes: 3E 02
0107H   CMP  E       ; 02H - 05H borrows: A is less  bytes: BB
```

| After       | `PC`    | `A`   | `E`   | `S` | `Z` | `AC` | `P` | `CY` |
|-------------|---------|-------|-------|-----|-----|------|-----|------|
| `MVI A,0AH` | `0102H` | `0AH` | `00H` | 0   | 0   | 0    | 0   | 0    |
| `MVI E,05H` | `0104H` | `0AH` | `05H` | 0   | 0   | 0    | 0   | 0    |
| `CMP E`     | `0105H` | `0AH` | `05H` | 0   | 0   | 1    | 1   | 0    |
| `MVI A,02H` | `0107H` | `02H` | `05H` | 0   | 0   | 1    | 1   | 0    |
| `CMP E`     | `0108H` | `02H` | `05H` | 1   | 0   | 0    | 0   | 1    |

### RLC, RRC, RAL, RAR: rotate A

`RLC` and the rest · opcode `00 0yy 111` · 1 byte · 4 cycles · flags: CY

Each rotate shifts `A` by one bit. The bit that falls off one end lands in
`CY`. The four differ in what enters the other end:

| Instruction | Opcode | Direction | Bit that enters              |
|-------------|--------|-----------|------------------------------|
| `RLC`       | `07H`  | left      | bit 7, the one that fell off |
| `RRC`       | `0FH`  | right     | bit 0, the one that fell off |
| `RAL`       | `17H`  | left      | the old `CY`                 |
| `RAR`       | `1FH`  | right     | the old `CY`                 |

```
RLC:   CY ← b7 ← b6 … b1 ← b0 ← b7       RAL:   CY ← b7 ← b6 … b1 ← b0 ← old CY
RRC:   b0 → b7 → b6 … b1 → b0 → CY       RAR:   old CY → b7 → b6 … b1 → b0 → CY
```

`RLC` and `RRC` rotate the 8 bits of `A` among themselves. `RAL` and `RAR`
rotate 9 bits, `A` and `CY` together, so a chain of them shifts a number
wider than 8 bits one byte at a time. The rotates leave `S`, `Z`, `AC` and
`P` alone.

The Intel manual gives one example for each: `RLC` turns `F2H` into `E5H`
with `CY = 1`, `RRC` turns `F2H` into `79H` with `CY = 0`. The program below
runs all four on one value.

```asm
0100H   MVI  A,81H   ; A = 1000 0001, CY = 0          bytes: 3E 81
0102H   RLC          ; A = 0000 0011, CY = 1          bytes: 07
0103H   RAL          ; A = 0000 0111, CY = 0          bytes: 17
0104H   RRC          ; A = 1000 0011, CY = 1          bytes: 0F
0105H   RAR          ; A = 1100 0001, CY = 1          bytes: 1F
```

| After       | `PC`    | `A`   | `A` in binary | `CY` |
|-------------|---------|-------|---------------|------|
| `MVI A,81H` | `0102H` | `81H` | `1000 0001`   | 0    |
| `RLC`       | `0103H` | `03H` | `0000 0011`   | 1    |
| `RAL`       | `0104H` | `07H` | `0000 0111`   | 0    |
| `RRC`       | `0105H` | `83H` | `1000 0011`   | 1    |
| `RAR`       | `0106H` | `C1H` | `1100 0001`   | 1    |

In `RAL`, bit 7 of `03H` is 0, so `CY` drops to 0, and the old `CY = 1`
fills bit 0. In `RAR`, the old `CY = 1` fills bit 7.

### CMA: complement A

`CMA` · opcode `2FH` · 1 byte · 4 cycles · flags: none

`CMA` inverts every bit of `A`, the same as `XRI 0FFH` but without touching
the flags. `CMA` followed by `INR A` negates `A` in two's complement.

The example repeats the one from the Intel manual.

```asm
0100H   MVI  A,51H   ; A = 0101 0001                  bytes: 3E 51
0102H   CMA          ; A = 1010 1110 = AEH            bytes: 2F
```

| After       | `PC`    | `A`   | `A` in binary |
|-------------|---------|-------|---------------|
| `MVI A,51H` | `0102H` | `51H` | `0101 0001`   |
| `CMA`       | `0103H` | `AEH` | `1010 1110`   |

### CMC: complement carry

`CMC` · opcode `3FH` · 1 byte · 4 cycles · flags: CY

`CMC` inverts `CY` and leaves everything else alone.

```asm
0100H   CMC          ; CY = 0 → 1                     bytes: 3F
0101H   CMC          ; CY = 1 → 0                     bytes: 3F
```

| After        | `PC`    | `CY` |
|--------------|---------|------|
| first `CMC`  | `0101H` | 1    |
| second `CMC` | `0102H` | 0    |

### STC: set carry

`STC` · opcode `37H` · 1 byte · 4 cycles · flags: CY

`STC` sets `CY` to 1 and leaves everything else alone. The 8080 has no
instruction that clears `CY` alone. Programs use `STC` followed by `CMC`,
or `ORA A`, which clears `CY` and sets the other flags from `A`.

```asm
0100H   STC          ; CY = 1                         bytes: 37
0101H   CMC          ; CY = 0                         bytes: 3F
```

| After | `PC`    | `CY` |
|-------|---------|------|
| `STC` | `0101H` | 1    |
| `CMC` | `0102H` | 0    |

### JMP: jump

`JMP nn` · opcode `C3H` · 3 bytes · 10 cycles · flags: none

`JMP` loads `nn` into `PC`, so the CPU fetches its next instruction from
there. The address follows the opcode low byte first: `JMP 0110H`
assembles to `C3 10 01`.

```asm
0100H   JMP  0110H   ; PC = 0110H                     bytes: C3 10 01
0103H   MVI  A,01H   ; skipped                        bytes: 3E 01
        ...
0110H   MVI  A,02H   ; A = 02H                        bytes: 3E 02
```

| After       | `PC`    | `A`   |
|-------------|---------|-------|
| `JMP 0110H` | `0110H` | `00H` |
| `MVI A,02H` | `0112H` | `02H` |

The 8080 also runs `CBH` as `JMP`. Intel never documented it.

### Jcc: jump if a condition holds

`Jcc nn` · opcode `11 ccc 010` · 3 bytes · 10 cycles · flags: none

`Jcc` jumps to `nn` if condition `cc` holds. Otherwise the CPU goes on to
the instruction after it. Each condition tests one flag. It takes 10 cycles
either way, since the CPU reads the address before it checks the flag.

| Instruction | Opcode | Jumps when | Instruction | Opcode | Jumps when |
|-------------|--------|------------|-------------|--------|------------|
| `JNZ nn`    | `C2H`  | `Z = 0`    | `JPO nn`    | `E2H`  | `P = 0`    |
| `JZ nn`     | `CAH`  | `Z = 1`    | `JPE nn`    | `EAH`  | `P = 1`    |
| `JNC nn`    | `D2H`  | `CY = 0`   | `JP nn`     | `F2H`  | `S = 0`    |
| `JC nn`     | `DAH`  | `CY = 1`   | `JM nn`     | `FAH`  | `S = 1`    |

The 3 bits `ccc` pick the condition. The same codes come back in the
conditional calls and returns.

`JNZ` after `DCR` makes a counted loop. The program below adds 5 three
times, so `A` ends at `0FH`, 15:

```asm
0100H   MVI  B,03H   ; B = 3, the loop counter        bytes: 06 03
0102H   MVI  A,00H   ; A = 0                          bytes: 3E 00
0104H   ADI  05H     ; A = A + 5                      bytes: C6 05
0106H   DCR  B       ; B = B - 1, Z = 1 when B is 0   bytes: 05
0107H   JNZ  0104H   ; back to ADI while B isn't 0    bytes: C2 04 01
010AH   HLT          ; stop                           bytes: 76
```

| After       | `PC`    | `A`   | `B`   | `Z` |
|-------------|---------|-------|-------|-----|
| `MVI B,03H` | `0102H` | `00H` | `03H` | 0   |
| `MVI A,00H` | `0104H` | `00H` | `03H` | 0   |
| `ADI 05H`   | `0106H` | `05H` | `03H` | 0   |
| `DCR B`     | `0107H` | `05H` | `02H` | 0   |
| `JNZ 0104H` | `0104H` | `05H` | `02H` | 0   |
| `ADI 05H`   | `0106H` | `0AH` | `02H` | 0   |
| `DCR B`     | `0107H` | `0AH` | `01H` | 0   |
| `JNZ 0104H` | `0104H` | `0AH` | `01H` | 0   |
| `ADI 05H`   | `0106H` | `0FH` | `01H` | 0   |
| `DCR B`     | `0107H` | `0FH` | `00H` | 1   |
| `JNZ 0104H` | `010AH` | `0FH` | `00H` | 1   |
| `HLT`       | `010BH` | `0FH` | `00H` | 1   |

The third `DCR B` sets `Z`, so the last `JNZ` falls through to `HLT`.

### CALL: call a subroutine

`CALL nn` · opcode `CDH` · 3 bytes · 17 cycles · flags: none

`CALL` pushes the return address, the address of the instruction after the
`CALL`, onto the stack and jumps to `nn`. The subroutine at `nn` ends with
`RET`, which pops the return address back into `PC`. Since the return
address sits on the stack, a subroutine can call another one.

The 8080 also runs `DDH`, `EDH` and `FDH` as `CALL`. Intel never documented
them.

```asm
0100H   LXI  SP,0F000H ; SP = F000H                   bytes: 31 00 F0
0103H   CALL 0110H    ; push 0106H, PC = 0110H        bytes: CD 10 01
0106H   HLT           ; runs after the RET            bytes: 76
        ...
0110H   MVI  A,42H    ; A = 42H                       bytes: 3E 42
0112H   RET           ; pop PC = 0106H                bytes: C9
```

| After           | `PC`    | `SP`    | `A`   | `memory[EFFEH]` | `memory[EFFFH]` |
|-----------------|---------|---------|-------|-----------------|-----------------|
| `LXI SP,0F000H` | `0103H` | `F000H` | `00H` | `00H`           | `00H`           |
| `CALL 0110H`    | `0110H` | `EFFEH` | `00H` | `06H`           | `01H`           |
| `MVI A,42H`     | `0112H` | `EFFEH` | `42H` | `06H`           | `01H`           |
| `RET`           | `0106H` | `F000H` | `42H` | `06H`           | `01H`           |
| `HLT`           | `0107H` | `F000H` | `42H` | `06H`           | `01H`           |

### Ccc: call if a condition holds

`Ccc nn` · opcode `11 ccc 100` · 3 bytes · 17 cycles if it calls, 11 if not
· flags: none

`Ccc` calls `nn` if condition `cc` holds, the way `CALL` does. Otherwise
the CPU goes on to the next instruction. The conditions and their codes
match `Jcc`. A call that doesn't happen skips the two stack writes, so it
takes 11 cycles instead of 17.

| Instruction | Opcode | Calls when | Instruction | Opcode | Calls when |
|-------------|--------|------------|-------------|--------|------------|
| `CNZ nn`    | `C4H`  | `Z = 0`    | `CPO nn`    | `E4H`  | `P = 0`    |
| `CZ nn`     | `CCH`  | `Z = 1`    | `CPE nn`    | `ECH`  | `P = 1`    |
| `CNC nn`    | `D4H`  | `CY = 0`   | `CP nn`     | `F4H`  | `S = 0`    |
| `CC nn`     | `DCH`  | `CY = 1`   | `CM nn`     | `FCH`  | `S = 1`    |

```asm
0100H   LXI  SP,0F000H ; SP = F000H                   bytes: 31 00 F0
0103H   MVI  A,0F0H   ; A = F0H                       bytes: 3E F0
0105H   ADI  20H      ; A = F0H + 20H = 10H, CY = 1   bytes: C6 20
0107H   CC   0110H    ; CY = 1, so call 0110H         bytes: DC 10 01
010AH   HLT           ; runs after the RET            bytes: 76
        ...
0110H   INR  B        ; B = B + 1                     bytes: 04
0111H   RET           ; pop PC = 010AH                bytes: C9
```

| After           | `PC`    | `SP`    | `A`   | `B`   | `CY` |
|-----------------|---------|---------|-------|-------|------|
| `LXI SP,0F000H` | `0103H` | `F000H` | `00H` | `00H` | 0    |
| `MVI A,0F0H`    | `0105H` | `F000H` | `F0H` | `00H` | 0    |
| `ADI 20H`       | `0107H` | `F000H` | `10H` | `00H` | 1    |
| `CC 0110H`      | `0110H` | `EFFEH` | `10H` | `00H` | 1    |
| `INR B`         | `0111H` | `EFFEH` | `10H` | `01H` | 1    |
| `RET`           | `010AH` | `F000H` | `10H` | `01H` | 1    |
| `HLT`           | `010BH` | `F000H` | `10H` | `01H` | 1    |

### RET: return from a subroutine

`RET` · opcode `C9H` · 1 byte · 10 cycles · flags: none

`RET` pops the two bytes at `SP` into `PC`, so the CPU continues after the
`CALL` that pushed them. The section on [`CALL`](#call-call-a-subroutine)
shows a full call and return.

The 8080 also runs `D9H` as `RET`. Intel never documented it.

Memory at `EFFEH` holds `06H` and at `EFFFH` holds `01H` before the
program runs, as a `CALL` at `0103H` would leave them.

```asm
0100H   LXI  SP,0EFFEH ; SP = EFFEH                   bytes: 31 FE EF
0103H   RET           ; pop PC = 0106H                bytes: C9
```

| After           | `PC`    | `SP`    |
|-----------------|---------|---------|
| `LXI SP,0EFFEH` | `0103H` | `EFFEH` |
| `RET`           | `0106H` | `F000H` |

### Rcc: return if a condition holds

`Rcc` · opcode `11 ccc 000` · 1 byte · 11 cycles if it returns, 5 if not ·
flags: none

`Rcc` returns if condition `cc` holds, the way `RET` does. Otherwise the CPU
goes on to the next instruction. A subroutine uses it to leave early. The
conditions and their codes match `Jcc`.

| Instruction | Opcode | Returns when | Instruction | Opcode | Returns when |
|-------------|--------|--------------|-------------|--------|--------------|
| `RNZ`       | `C0H`  | `Z = 0`      | `RPO`       | `E0H`  | `P = 0`      |
| `RZ`        | `C8H`  | `Z = 1`      | `RPE`       | `E8H`  | `P = 1`      |
| `RNC`       | `D0H`  | `CY = 0`     | `RP`        | `F0H`  | `S = 0`      |
| `RC`        | `D8H`  | `CY = 1`     | `RM`        | `F8H`  | `S = 1`      |

The subroutine below returns early when `A` is 0, so `MVI B,01H` never
runs.

```asm
0100H   LXI  SP,0F000H ; SP = F000H                   bytes: 31 00 F0
0103H   MVI  A,00H    ; A = 00H                       bytes: 3E 00
0105H   CALL 0110H    ; push 0108H, PC = 0110H        bytes: CD 10 01
0108H   HLT           ; runs after the RZ             bytes: 76
        ...
0110H   ORA  A        ; Z = 1, since A = 0            bytes: B7
0111H   RZ            ; Z = 1, so pop PC = 0108H      bytes: C8
0112H   MVI  B,01H    ; skipped                       bytes: 06 01
0114H   RET                                           bytes: C9
```

| After           | `PC`    | `SP`    | `B`   | `Z` |
|-----------------|---------|---------|-------|-----|
| `LXI SP,0F000H` | `0103H` | `F000H` | `00H` | 0   |
| `MVI A,00H`     | `0105H` | `F000H` | `00H` | 0   |
| `CALL 0110H`    | `0110H` | `EFFEH` | `00H` | 0   |
| `ORA A`         | `0111H` | `EFFEH` | `00H` | 1   |
| `RZ`            | `0108H` | `F000H` | `00H` | 1   |
| `HLT`           | `0109H` | `F000H` | `00H` | 1   |

### RST: restart

`RST n` · opcode `11 nnn 111` · 1 byte · 11 cycles · flags: none

`RST n` calls the subroutine at address `8 × n`, for `n` from 0 to 7. It
works like a one-byte `CALL` to one of eight fixed addresses.

| Instruction | Opcode | Calls   | Instruction | Opcode | Calls   |
|-------------|--------|---------|-------------|--------|---------|
| `RST 0`     | `C7H`  | `0000H` | `RST 4`     | `E7H`  | `0020H` |
| `RST 1`     | `CFH`  | `0008H` | `RST 5`     | `EFH`  | `0028H` |
| `RST 2`     | `D7H`  | `0010H` | `RST 6`     | `F7H`  | `0030H` |
| `RST 3`     | `DFH`  | `0018H` | `RST 7`     | `FFH`  | `0038H` |

An interrupting device can supply any 8080 instruction. Most supply an
`RST` opcode on the data bus, which calls a handler at one of the eight
addresses. The CPU leaves `PC` at the interrupted program's next
instruction, so `RET` resumes there. `RST 0` jumps to `0000H`, where the CPU
starts after a reset.

The [Intel 8080 Assembly Language Programming Manual](https://altairclone.com/downloads/manuals/8080%20Programmers%20Manual.pdf),
chapter 5, describes this sequence. `i8080::Cpu::interrupt` takes the
instruction bytes from a callback; the machine calls it between CPU steps.

```asm
0100H   LXI  SP,0F000H ; SP = F000H                   bytes: 31 00 F0
0103H   RST  3        ; push 0104H, PC = 0018H        bytes: DF
```

| After           | `PC`    | `SP`    | `memory[EFFEH]` | `memory[EFFFH]` |
|-----------------|---------|---------|-----------------|-----------------|
| `LXI SP,0F000H` | `0103H` | `F000H` | `00H`           | `00H`           |
| `RST 3`         | `0018H` | `EFFEH` | `04H`           | `01H`           |

### PCHL: jump to the address in HL

`PCHL` · opcode `E9H` · 1 byte · 5 cycles · flags: none

`PCHL` copies `HL` into `PC`. A program can compute the address in `HL`
first, for example to pick an entry from a table of jump targets.

The example repeats the one from the Intel manual.

```asm
0100H   LXI  H,413EH ; HL = 413EH                     bytes: 21 3E 41
0103H   PCHL         ; PC = HL = 413EH                bytes: E9
```

| After         | `PC`    | `HL`    |
|---------------|---------|---------|
| `LXI H,413EH` | `0103H` | `413EH` |
| `PCHL`        | `413EH` | `413EH` |

### PUSH: push onto the stack

`PUSH rp` · opcode `11 rp0 101` · 1 byte · 11 cycles · flags: none

The stack is a region of memory that `SP` points into. It grows down:
`PUSH` lowers `SP` by 2 and stores a register pair in the two bytes there,
the high byte at `SP + 1` and the low byte at `SP`. Programs set `SP` once
with `LXI SP` and then save registers with `PUSH` and restore them with
`POP`.

| Instruction | Opcode | Stores                   |
|-------------|--------|--------------------------|
| `PUSH B`    | `C5H`  | `B`, then `C`            |
| `PUSH D`    | `D5H`  | `D`, then `E`            |
| `PUSH H`    | `E5H`  | `H`, then `L`            |
| `PUSH PSW`  | `F5H`  | `A`, then the flags byte |

Code `11` means `PSW` here, the program status word: `A` and the flags.
`PUSH SP` doesn't exist. The flags byte packs the five flags like this:

| Bit  | 7   | 6   | 5 | 4    | 3 | 2   | 1 | 0    |
|------|-----|-----|---|------|---|-----|---|------|
| Flag | `S` | `Z` | 0 | `AC` | 0 | `P` | 1 | `CY` |

Bits 5 and 3 are always 0 and bit 1 is always 1. With `Z`, `AC`, `P` and
`CY` at 1, the flags byte is `57H`.

The example repeats the one from the Intel manual and then pops the value
into another pair.

```asm
0100H   LXI  SP,3A2CH ; SP = 3A2CH                    bytes: 31 2C 3A
0103H   LXI  D,8F9DH  ; DE = 8F9DH                    bytes: 11 9D 8F
0106H   PUSH D        ; memory[3A2BH] = D = 8FH,      bytes: D5
                      ; memory[3A2AH] = E = 9DH
0107H   POP  B        ; BC = 8F9DH                    bytes: C1
```

| After          | `PC`    | `SP`    | `BC`    | `DE`    | `memory[3A2AH]` | `memory[3A2BH]` |
|----------------|---------|---------|---------|---------|-----------------|-----------------|
| `LXI SP,3A2CH` | `0103H` | `3A2CH` | `0000H` | `0000H` | `00H`           | `00H`           |
| `LXI D,8F9DH`  | `0106H` | `3A2CH` | `0000H` | `8F9DH` | `00H`           | `00H`           |
| `PUSH D`       | `0107H` | `3A2AH` | `0000H` | `8F9DH` | `9DH`           | `8FH`           |
| `POP B`        | `0108H` | `3A2CH` | `8F9DH` | `8F9DH` | `9DH`           | `8FH`           |

`POP` leaves the bytes in memory. Only `SP` moves.

### POP: pop from the stack

`POP rp` · opcode `11 rp0 001` · 1 byte · 10 cycles · flags: all, for
`POP PSW`

`POP` loads a register pair from the two bytes at `SP`, the low byte from
`SP` and the high byte from `SP + 1`, and raises `SP` by 2. It undoes a
`PUSH`.

| Instruction | Opcode | Loads                    |
|-------------|--------|--------------------------|
| `POP B`     | `C1H`  | `C`, then `B`            |
| `POP D`     | `D1H`  | `E`, then `D`            |
| `POP H`     | `E1H`  | `L`, then `H`            |
| `POP PSW`   | `F1H`  | the flags byte, then `A` |

`POP PSW` sets all five flags from the flags byte and ignores bits 5, 3
and 1.

The example repeats the one from the Intel manual. Memory at `1239H` holds
`3DH` and at `123AH` holds `93H` before the program runs.

```asm
0100H   LXI  SP,1239H ; SP = 1239H                    bytes: 31 39 12
0103H   POP  H        ; L = 3DH, H = 93H              bytes: E1
```

| After          | `PC`    | `SP`    | `HL`    |
|----------------|---------|---------|---------|
| `LXI SP,1239H` | `0103H` | `1239H` | `0000H` |
| `POP H`        | `0104H` | `123BH` | `933DH` |

`POP PSW` on the same bytes would load `A = 93H` and the flags from
`3DH = 0011 1101`: `AC`, `P` and `CY` at 1, `S` and `Z` at 0.

### XTHL: swap HL with the top of the stack

`XTHL` · opcode `E3H` · 1 byte · 18 cycles · flags: none

`XTHL` swaps `L` with the byte at `SP` and `H` with the byte at `SP + 1`.
`SP` stays as it was. It takes 18 cycles, the most of any 8080
instruction, for two memory reads and two writes.

The example repeats the one from the Intel manual. Memory at `10ADH` holds
`F0H` and at `10AEH` holds `0DH` before the program runs.

```asm
0100H   LXI  SP,10ADH ; SP = 10ADH                    bytes: 31 AD 10
0103H   LXI  H,0B3CH  ; HL = 0B3CH                    bytes: 21 3C 0B
0106H   XTHL          ; HL = 0DF0H, stack = 3CH 0BH   bytes: E3
```

| After          | `PC`    | `SP`    | `HL`    | `memory[10ADH]` | `memory[10AEH]` |
|----------------|---------|---------|---------|-----------------|-----------------|
| `LXI SP,10ADH` | `0103H` | `10ADH` | `0000H` | `F0H`           | `0DH`           |
| `LXI H,0B3CH`  | `0106H` | `10ADH` | `0B3CH` | `F0H`           | `0DH`           |
| `XTHL`         | `0107H` | `10ADH` | `0DF0H` | `3CH`           | `0BH`           |

### SPHL: copy HL into SP

`SPHL` · opcode `F9H` · 1 byte · 5 cycles · flags: none

`SPHL` copies `HL` into `SP`. A program can compute where its stack goes,
for example at the top of the RAM it found, and move `SP` there.

The example repeats the one from the Intel manual.

```asm
0100H   LXI  H,506CH  ; HL = 506CH                    bytes: 21 6C 50
0103H   SPHL          ; SP = HL = 506CH               bytes: F9
```

| After         | `PC`    | `SP`    | `HL`    |
|---------------|---------|---------|---------|
| `LXI H,506CH` | `0103H` | `0000H` | `506CH` |
| `SPHL`        | `0104H` | `506CH` | `506CH` |

### IN: read an I/O port

`IN n` · opcode `DBH` · 2 bytes · 10 cycles · flags: none

`IN` reads one byte from I/O port `n` into `A`. The 8080 has 256 ports,
separate from memory, and the byte after the opcode picks one. The machine
decides what sits behind each port: on the Altair, the 88-2SIO serial card
answers at ports `10H` (status) and `11H` (data).

The program below waits for a key from the terminal. It reads the status
port until bit 0, "data ready", turns 1, and then reads the key from the
data port. The table assumes the key `A` (`41H`) arrives between the first
and second status read.

```asm
0100H   IN   10H     ; A = serial status              bytes: DB 10
0102H   ANI  01H     ; keep bit 0, Z = 1 if no key    bytes: E6 01
0104H   JZ   0100H   ; no key yet: read status again  bytes: CA 00 01
0107H   IN   11H     ; A = the key                    bytes: DB 11
```

| After      | `PC`    | `A`   | `Z` |
|------------|---------|-------|-----|
| `IN 10H`   | `0102H` | `00H` | 0   |
| `ANI 01H`  | `0104H` | `00H` | 1   |
| `JZ 0100H` | `0100H` | `00H` | 1   |
| `IN 10H`   | `0102H` | `01H` | 1   |
| `ANI 01H`  | `0104H` | `01H` | 0   |
| `JZ 0100H` | `0107H` | `01H` | 0   |
| `IN 11H`   | `0109H` | `41H` | 0   |

### OUT: write an I/O port

`OUT n` · opcode `D3H` · 2 bytes · 10 cycles · flags: none

`OUT` writes `A` to I/O port `n`. On the Altair, a byte written to port
`11H` goes out of the 88-2SIO serial card to the terminal.

```asm
0100H   MVI  A,41H   ; A = 41H, the character "A"     bytes: 3E 41
0102H   OUT  11H     ; send A to the terminal         bytes: D3 11
```

| After       | `PC`    | `A`   | Written to port `11H` |
|-------------|---------|-------|-----------------------|
| `MVI A,41H` | `0102H` | `41H` | nothing               |
| `OUT 11H`   | `0104H` | `41H` | `41H`                 |

### EI: enable interrupts

`EI` · opcode `FBH` · 1 byte · 4 cycles · flags: none

`EI` turns on the CPU's interrupt enable, which the 8080 shows on its INTE
pin. With interrupts enabled, a device can stop the running program, make
the CPU run an `RST` and so call a handler. The CPU turns interrupts off
when it accepts one, and the handler ends with `EI` and `RET` to turn them
back on. The real 8080 waits one more instruction before it accepts an
interrupt after `EI`, so the handler's `RET` runs first.

The 8080 starts with interrupts disabled.

```asm
0100H   EI           ; interrupts on                  bytes: FB
0101H   DI           ; interrupts off                 bytes: F3
```

| After | `PC`    | Interrupts |
|-------|---------|------------|
| `EI`  | `0101H` | enabled    |
| `DI`  | `0102H` | disabled   |

**Emulator status:** `Cpu::interrupt` observes the one-instruction wait
after `EI` and clears INTE when it accepts an interrupt. The Altair's 88-2SIO
can request `RST 7` through PINT; see
[serial interrupts](../altair/memory-and-ports.md#serial-interrupts).

### DI: disable interrupts

`DI` · opcode `F3H` · 1 byte · 4 cycles · flags: none

`DI` turns the interrupt enable off. Devices can still ask for an interrupt,
but the CPU ignores them until the next `EI`. A program uses `DI` around
code that an interrupt handler must not break into, such as an update of a
value the handler also reads. The section on [`EI`](#ei-enable-interrupts)
shows both.

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

**Emulator status:** an interrupt that `Cpu::interrupt` accepts wakes the
CPU. With INTE clear, the CPU stays halted until RESET.

### NOP: do nothing

`NOP` · opcode `00H` · 1 byte · 4 cycles · flags: none

`NOP` changes no register, flag or memory byte. The CPU spends 4 cycles
fetching it and moves `PC` to the next byte. Programmers used it to fill
space, reserve bytes for a later patch or wait a few microseconds.

Zeroed memory reads as `NOP`, so a CPU running through empty RAM executes
`NOP` after `NOP` until it reaches something else.

Intel documents only `00H`. The 8080 also runs `08H`, `10H`, `18H`, `20H`,
`28H`, `30H` and `38H` as `NOP`: they share the `00 yyy 000` column, and
the CPU ignores `yyy` there. The emulator does the same.

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
