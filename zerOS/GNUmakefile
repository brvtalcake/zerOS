# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

override KERNEL := zerOS
override KERNEL_SECTIONS := text bootcode rodata data bss # dynamic

override COMMON_DEPS := GNUmakefile sectioninfo config

include gnumake/gmsl

# TODO: Rebuild a GNU Make with guile support.

include gnumake/misc.mk
include gnumake/default_kvars.mk
include gnumake/flags.mk
include gnumake/prerequisites.mk

override CFILES := $(shell cd src && find -L * -type f -name '*.c')
override KLIBC_CFILES := $(addprefix klibc/,$(shell cd src/klibc && find -L * -type f -name '*.c'))
override KERNEL_CFILES := $(filter-out $(KLIBC_CFILES),$(CFILES))

override ASFILES := $(shell cd src && find -L * -type f -name '*.s')
override KLIBC_ASFILES := $(addprefix klibc/,$(shell cd src/klibc && find -L * -type f -name '*.s'))
override KERNEL_ASFILES := $(filter-out $(KLIBC_ASFILES),$(ASFILES))

override GCCASFILES := $(shell cd src && find -L * -type f -name '*.S')
override KLIBC_GCCASFILES := $(addprefix klibc/,$(shell cd src/klibc && find -L * -type f -name '*.S'))
override KERNEL_GCCASFILES := $(filter-out $(KLIBC_GCCASFILES),$(GCCASFILES))

override NASMFILES := $(shell cd src && find -L * -type f -name '*.asm')
override KLIBC_NASMFILES := $(addprefix klibc/,$(shell cd src/klibc && find -L * -type f -name '*.asm'))
override KERNEL_NASMFILES := $(filter-out $(KLIBC_NASMFILES),$(NASMFILES))

override OBJ := $(addprefix obj/,$(CFILES:.c=.c.o) $(ASFILES:.s=.s.o) $(NASMFILES:.asm=.asm.o) $(GCCASFILES:.S=.S.o))
override KLIBC_OBJ := $(addprefix obj/,$(KLIBC_CFILES:.c=.c.o) $(KLIBC_ASFILES:.s=.s.o) $(KLIBC_NASMFILES:.asm=.asm.o) $(KLIBC_GCCASFILES:.S=.S.o))
override KERNEL_OBJ := $(addprefix obj/,$(KERNEL_CFILES:.c=.c.o) $(KERNEL_ASFILES:.s=.s.o) $(KERNEL_NASMFILES:.asm=.asm.o) $(KERNEL_GCCASFILES:.S=.S.o))

override HEADER_DEPS := $(addprefix obj/,$(CFILES:.c=.c.d) $(GCCASFILES:.S=.S.d))
override KERNEL_SUPDEF_CONFIG := include/config.sd
override CONFIG_DEPS := $(shell find -L include/config.d/ -type f)

override KERNEL_MAP := linker-$(KARCH).ld

include gnumake/printvar.mk

$(eval $(call PRINTVAR_RULE,KERNEL))
$(eval $(call PRINTVAR_RULE,CFILES))
$(eval $(call PRINTVAR_RULE,KLIBC_CFILES))
$(eval $(call PRINTVAR_RULE,KERNEL_CFILES))
$(eval $(call PRINTVAR_RULE,ASFILES))
$(eval $(call PRINTVAR_RULE,KLIBC_ASFILES))
$(eval $(call PRINTVAR_RULE,KERNEL_ASFILES))
$(eval $(call PRINTVAR_RULE,GCCASFILES))
$(eval $(call PRINTVAR_RULE,KLIBC_GCCASFILES))
$(eval $(call PRINTVAR_RULE,KERNEL_GCCASFILES))
$(eval $(call PRINTVAR_RULE,NASMFILES))
$(eval $(call PRINTVAR_RULE,KLIBC_NASMFILES))
$(eval $(call PRINTVAR_RULE,KERNEL_NASMFILES))
$(eval $(call PRINTVAR_RULE,OBJ))
$(eval $(call PRINTVAR_RULE,KLIBC_OBJ))
$(eval $(call PRINTVAR_RULE,KERNEL_OBJ))
$(eval $(call PRINTVAR_RULE,HEADER_DEPS))
$(eval $(call PRINTVAR_RULE,KERNEL_MAP))

$(eval $(call PRINTVAR_RULE,BOOT_KCFLAGS))
$(eval $(call PRINTVAR_RULE,KCFLAGS))
$(eval $(call PRINTVAR_RULE,KCPPFLAGS))
$(eval $(call PRINTVAR_RULE,KLDFLAGS))

# Default target: build the kernel.

.PHONY: all sectioninfo iso config

config: include/config.h

sectioninfo: GNUmakefile $(KERNEL_MAP) include/kernel/generated/sections.h

all: bin/$(KERNEL)

iso: bin/$(KERNEL).iso	

bin/$(KERNEL).iso: bin/$(KERNEL) $(COMMON_DEPS) $(LIMINE_CFG)
	mkdir -p "$$(dirname $@)"
	mkdir -p iso_root/boot/limine
	cp bin/$(KERNEL) iso_root/boot/
	cp $(LIMINE_CFG) iso_root/boot/limine/
	cp $(LIMINE_DATADIR)/*.sys iso_root/boot/limine/
	cp $(LIMINE_DATADIR)/*.bin iso_root/boot/limine/
	mkdir -p iso_root/EFI/BOOT
	cp $(LIMINE_DATADIR)/BOOTX64.EFI iso_root/EFI/BOOT/
	cp $(LIMINE_DATADIR)/BOOTIA32.EFI iso_root/EFI/BOOT/
	$(XORRISO) -as mkisofs -b boot/limine/limine-bios-cd.bin 		\
        -no-emul-boot -boot-load-size 4 -boot-info-table 			\
        --efi-boot boot/limine/limine-uefi-cd.bin 					\
        -efi-boot-part --efi-boot-image --protective-msdos-label 	\
        iso_root -o $@
	$(LIMINE) bios-install $@

bin/$(KERNEL): $(COMMON_DEPS) $(OBJ)
	mkdir -p "$$(dirname $@)"
	$(KLD) $(KLDFLAGS) $(OBJ) -o $@
	printf '\003' | dd of=$@ bs=1 count=1 seek=16 conv=notrunc 2>/dev/null
	$(if $(call not,$(DEBUG_KERNEL)), $(KSTRIP) --strip-debug $@)

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
obj/%.c.o: src/%.c $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@
obj/boot/%.c.o: src/boot/%.c $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@

# Compilation rules for *.s files.
obj/%.s.o: src/%.s $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KAS) $(KASFLAGS) $< -o $@
obj/boot/%.s.o: src/boot/%.s $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KAS) $(BOOT_KASFLAGS) $< -o $@

# Compilation rules for *.S files.
obj/%.S.o: src/%.S $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@
obj/boot/%.S.o: src/boot/%.S $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@

# Compilation rules for *.asm (nasm) files.
obj/%.asm.o: src/%.asm $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KNASM) $(KNASMFLAGS) $< -o $@
obj/boot/%.asm.o: src/boot/%.asm $(COMMON_DEPS)
	mkdir -p "$$(dirname $@)"
	$(KNASM) $(KNASMFLAGS) $< -o $@

override define GENSECINFO =
mkdir -p "$$(dirname include/kernel/generated/sections.h)"
../scripts/gensectioninfo.py 						\
--output-cfile include/kernel/generated/sections.h 	\
--output-linkerfile $(KERNEL_MAP) 					\
--input-linkerfile $(KERNEL_MAP).template 			\
$(KERNEL_SECTIONS)
endef

$(KERNEL_MAP): GNUmakefile $(KERNEL_MAP).template ../scripts/gensectioninfo.py
	$(GENSECINFO)

include/kernel/generated/sections.h: GNUmakefile $(KERNEL_MAP).template ../scripts/gensectioninfo.py
	mkdir -p "$$(dirname $@)"
	$(GENSECINFO)

override KCPU_FEATURES := $(shell cpuid_tool --flags | grep --invert-match -e '-')
$(eval $(call PRINTVAR_RULE,KCPU_FEATURES))

override define GENCONFIG =
@echo "/* Automatically generated. Do not edit. */" > $@
@echo "#ifndef zerOS_CONFIG_H_INCLUDED" >> $@
@echo "#define zerOS_CONFIG_H_INCLUDED" >> $@
@echo "" >> $@
@echo "#undef  zerOS_CONFIG_CPU" >> $@
@echo "#define zerOS_CONFIG_CPU $(KCPU)" >> $@
@echo "" >> $@
@echo "#undef  zerOS_CONFIG_ARCH" >> $@
@echo "#define zerOS_CONFIG_ARCH $(KARCH)" >> $@
@echo "" >> $@
@echo "#undef  zerOS_CONFIG_DEBUG" >> $@
@echo "#define zerOS_CONFIG_DEBUG $(if $(DEBUG_KERNEL),true,false)" >> $@
@echo "" >> $@
@echo "#undef  zerOS_CONFIG_CPU_FEATURES" >> $@
@echo "#define zerOS_CONFIG_CPU_FEATURES $(KCPU_FEATURES)" >> $@
@echo "" >> $@
@cat  $(KERNEL_SUPDEF_CONFIG) >> $@
@echo "" >> $@
@echo "#endif" >> $@
endef

override define SUPDEF =
../scripts/supdef.py $< -o $@ -Iinclude/config.d/
endef

include/config.h: include/config.h.in GNUmakefile
	mkdir -p "$$(dirname $@)"
	$(SUPDEF)

include/config.h.in: GNUmakefile $(CONFIG_DEPS)  $(KERNEL_SUPDEF_CONFIG)
	mkdir -p "$$(dirname $@)"
	$(GENCONFIG)

# Remove object files and the final executable.
.PHONY: clean
clean:
	rm -rf bin obj include/kernel/generated/sections.h iso_root $(KERNEL_MAP)