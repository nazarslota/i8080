# Builds the 8080 assembly programs in this folder into CP/M .COM files and
# runs them in the emulator. Each program lives in a folder of its own,
# such as main/main.asm. docs/guide/cpm.md explains the steps.
#
#   make                       build every program into a .COM file
#   make run                   build main, copy it to b.dsk, boot CP/M
#   make run PROGRAM=name      the same for name/name.asm
#   make clean                 remove the .COM files

ROOT     := ../..
ALTAIR   := cargo run --release --
Z80ASM   := docker run --rm -v "$(CURDIR)":/src -w /src z88dk/z88dk z88dk-z80asm -m8080 -b
PROGRAM  := main
PROGRAMS := $(patsubst %.asm,%.COM,$(wildcard */*.asm))

.PHONY: asm run clean

asm: $(PROGRAMS)

# z80asm leaves an object file next to the binary; the binary is all CP/M
# needs.
%.COM: %.asm
	$(Z80ASM) -o$@ $<
	rm -f $*.o

run: $(PROGRAM)/$(PROGRAM).COM $(ROOT)/a.dsk $(ROOT)/b.dsk
	$(ALTAIR) disk put $(ROOT)/b.dsk $<
	$(ALTAIR) cpm --prom $(ROOT)/roms/DBL.HEX $(ROOT)/a.dsk $(ROOT)/b.dsk

$(ROOT)/a.dsk:
	cp $(ROOT)/roms/cpm22b23-56k.dsk $@

$(ROOT)/b.dsk:
	$(ALTAIR) disk format $@

clean:
	rm -f */*.COM
