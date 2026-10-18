# Builds the C programs in examples/c for CP/M on the 8080 and runs them in
# the emulator. Cargo builds the Rust code. docs/guide/cpm.md explains the
# steps.
#
#   make                       build every examples/c/*.c into a .COM file
#   make run                   build main.c, copy it to b.dsk, boot CP/M
#   make run PROGRAM=name      the same for examples/c/name.c
#   make clean                 remove the .COM files

ALTAIR   := cargo run --release --
ZCC      := docker run --rm -v "$(CURDIR)":/src -w /src z88dk/z88dk zcc +cpm -clib=8080
PROGRAM  := main
PROGRAMS := $(patsubst %.c,%.COM,$(wildcard examples/c/*.c))

.PHONY: c run clean

c: $(PROGRAMS)

examples/c/%.COM: examples/c/%.c
	$(ZCC) $< -o $@

run: examples/c/$(PROGRAM).COM a.dsk b.dsk
	$(ALTAIR) disk put b.dsk $<
	$(ALTAIR) cpm a.dsk b.dsk

a.dsk:
	cp roms/cpm22b23-56k.dsk $@

b.dsk:
	$(ALTAIR) disk format $@

clean:
	rm -f examples/c/*.COM
