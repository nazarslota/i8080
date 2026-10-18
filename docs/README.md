# Documentation

| Page                                                     | What it covers                                        |
|----------------------------------------------------------|-------------------------------------------------------|
| [guide/basic.md](guide/basic.md)                         | running Altair BASIC from its tape                    |
| [guide/cpm.md](guide/cpm.md)                             | running CP/M, extra disks, copying programs onto them |
| [architecture.md](architecture.md)                       | the layers, their rules and where each part lives     |
| [boot.md](boot.md)                                       | the loaders, from RUN to `OK` or `A>`                 |
| [altair/front-panel.md](altair/front-panel.md)           | the switches and lights                               |
| [altair/memory-and-ports.md](altair/memory-and-ports.md) | the memory map and the I/O ports                      |
| [i8080/instructions.md](i8080/instructions.md)           | every 8080 instruction, with opcodes and examples     |
| [glossary.md](glossary.md)                               | the terms                                             |

The guides tell you how to use the emulator. The other pages explain the
hardware and the code; read [architecture.md](architecture.md) before you
change code. `cargo doc --open` documents each type and function.
