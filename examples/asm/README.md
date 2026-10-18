# Assembly programs for CP/M

8080 assembly programs that run under CP/M on the emulated Altair. Each
program lives in a folder of its own, with a source file of the same name:
`main/main.asm`. In this folder, `make run PROGRAM=name` assembles
`name/name.asm` with z80asm and runs it;
[docs/guide/cpm.md](../../docs/guide/cpm.md#assembly-programs) explains
the steps.

| Program | What it does                            |
|---------|-----------------------------------------|
| `main`  | prints a greeting through the CP/M BDOS |
