# Assembly programs for CP/M

8080 assembly programs that the CP/M assembler, `ASM.COM`, builds on the
emulated Altair. Each program lives in a folder of its own, with a source
file of the same name: `main/main.asm`. In this folder,
`make run PROGRAM=name` puts `name/name.asm` on `b.dsk` and boots CP/M;
[docs/guide/cpm.md](../../docs/guide/cpm.md#assembly-programs) shows the
commands that assemble and run it.

| Program | What it does                                |
|---------|---------------------------------------------|
| `main`  | prints a greeting through the CP/M BDOS     |
