# C programs for CP/M

C programs that run under CP/M on the emulated Altair. Each program lives
in a folder of its own, with a source file of the same name:
`main/main.c`. In this folder, `make run PROGRAM=name` compiles
`name/name.c` with z88dk and runs it;
[docs/guide/cpm.md](../docs/guide/cpm.md#c-programs) explains the steps.

| Program | What it does                                     |
|---------|--------------------------------------------------|
| `main`  | a program to start from: `make run`              |
