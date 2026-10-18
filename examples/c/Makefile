# Builds the C programs in this folder for CP/M on the 8080 and runs them in
# the emulator. Each program lives in a folder of its own, such as
# main/main.c. docs/guide/cpm.md explains the steps.
#
#   make                       build every program into a .COM file
#   make run                   build main, copy it to b.dsk, boot CP/M
#   make run PROGRAM=name      the same for name/name.c
#   make clean                 remove the .COM files

ROOT     := ../..
ALTAIR   := cargo run --release --
ZCC      := docker run --rm -v "$(CURDIR)":/src -w /src z88dk/z88dk zcc +cpm -clib=8080 -lm
PROGRAM  := main
PROGRAMS := $(patsubst %.c,%.COM,$(wildcard */*.c))

.PHONY: c run clean

c: $(PROGRAMS)

%.COM: %.c
	$(ZCC) $< -o $@

run: $(PROGRAM)/$(PROGRAM).COM $(ROOT)/a.dsk $(ROOT)/b.dsk
	$(ALTAIR) disk put $(ROOT)/b.dsk $<
	$(ALTAIR) cpm --prom $(ROOT)/roms/DBL.HEX $(ROOT)/a.dsk $(ROOT)/b.dsk

$(ROOT)/a.dsk:
	cp $(ROOT)/roms/cpm22b23-56k.dsk $@

$(ROOT)/b.dsk:
	$(ALTAIR) disk format $@

clean:
	rm -f */*.COM
