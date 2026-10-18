# Puts the 8080 assembly programs in this folder on a CP/M disk, where the
# CP/M assembler builds them. Each program lives in a folder of its own,
# such as main/main.asm. docs/guide/cpm.md explains the steps.
#
#   make run                   put main.asm on b.dsk, boot CP/M
#   make run PROGRAM=name      the same for name/name.asm
#   make clean                 remove the build folder
#
# At A>, assemble and run the program with the tools of CP/M:
#
#   B:
#   A:ASM MAIN
#   A:LOAD MAIN
#   MAIN

ROOT     := ../..
ALTAIR   := cargo run --release --
PROGRAM  := main

.PHONY: run clean

run: build/$(PROGRAM).asm $(ROOT)/a.dsk $(ROOT)/b.dsk
	$(ALTAIR) disk put $(ROOT)/b.dsk $<
	$(ALTAIR) cpm --prom $(ROOT)/roms/DBL.HEX $(ROOT)/a.dsk $(ROOT)/b.dsk

# CP/M ends each line of text with CR LF, and ASM reads a file with bare LF
# as empty. The copy in build/ gets the CR LF.
build/$(PROGRAM).asm: $(PROGRAM)/$(PROGRAM).asm
	mkdir -p build
	awk '{ sub(/\r$$/, ""); printf "%s\r\n", $$0 }' $< > $@

$(ROOT)/a.dsk:
	cp $(ROOT)/roms/cpm22b23-56k.dsk $@

$(ROOT)/b.dsk:
	$(ALTAIR) disk format $@

clean:
	rm -rf build
