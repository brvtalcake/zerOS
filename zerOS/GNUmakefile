# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

override KERNEL := zerOS

include gnumake/gmsl

# TODO: Rebuild a GNU Make with guile support.

include gnumake/misc.mk
include gnumake/default_kvars.mk
include gnumake/flags.mk
include gnumake/prerequisites.mk

override CFILES := $(shell cd src && find -L * -type f -name '*.c')
override ASFILES := $(shell cd src && find -L * -type f -name '*.S')
override NASMFILES := $(shell cd src && find -L * -type f -name '*.asm')
override OBJ := $(addprefix obj/,$(CFILES:.c=.c.o) $(ASFILES:.S=.S.o) $(NASMFILES:.asm=.asm.o))
override HEADER_DEPS := $(addprefix obj/,$(CFILES:.c=.c.d) $(ASFILES:.S=.S.d))

override KERNEL_MAP := linker-$(KARCH).ld

.PHONY: all
all: bin/$(KERNEL)

bin/$(KERNEL): GNUmakefile $(KERNEL_MAP) $(OBJ)
	mkdir -p "$$(dirname $@)"
	$(KLD) $(OBJ) $(KLDFLAGS) -o $@
	printf '\003' | dd of=$@ bs=1 count=1 seek=16 conv=notrunc 2>/dev/null

# Include header dependencies.
-include $(HEADER_DEPS)

# Generate header dependencies.
obj/%.c.d: src/%.c GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KPP) $(KCFLAGS) $(KCPPFLAGS) -M -MG -MP -MT $(@:.d=.o) $< -MF $@ > /dev/null
obj/boot/%.c.d: src/boot/%.c GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KPP) $(BOOT_KCFLAGS) $(KCPPFLAGS) -M -MG -MP -MT $(@:.d=.o) $< -MF $@ > /dev/null

# Compilation rules for *.c files.
obj/%.c.o: src/%.c GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@
obj/boot/%.c.o: src/boot/%.c GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@
#obj/klibc/string/%.c.o: src/klibc/string/%.c GNUmakefile
#	mkdir -p "$$(dirname $@)"
#	$(KCC) $(KCFLAGS) -minline-all-stringops $(KCPPFLAGS) -c $< -o $@

# Compilation rules for *.S files.
obj/%.S.o: src/%.S GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@
obj/boot/%.S.o: src/boot/%.S GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@

# Compilation rules for *.asm (nasm) files.
obj/%.asm.o: src/%.asm GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KNASM) $(KNASMFLAGS) $< -o $@
obj/boot/%.asm.o: src/boot/%.asm GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(KNASM) $(KNASMFLAGS) $< -o $@

# Remove object files and the final executable.
.PHONY: clean
clean:
	rm -rf bin obj