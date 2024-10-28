# Nuke built-in rules and variables.
override MAKEFLAGS += -rR

SHELL := /bin/bash

override CURRENT_MAKEFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
override CURRENT_MAKEFILE_DIR  := $(dir $(CURRENT_MAKEFILE_PATH))

override KERNEL_LOG_DIR := $(CURRENT_MAKEFILE_DIR)/logs/

override KERNEL := zerOS
override KERNEL_SECTIONS := text bootcode rodata data bss # dynamic

override MAKEFILE_DEPS := $(shell find -L gnumake -type f -name '*.mk')

override COMMON_DEPS := GNUmakefile $(KERNEL_MAP) $(MAKEFILE_DEPS)

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

# Default target: build the kernel.
.PHONY: all sectioninfo iso config

.DEFAULT_GOAL := all

$(eval $(call PRINTVAR_RULE,CURRENT_MAKEFILE_PATH))
$(eval $(call PRINTVAR_RULE,CURRENT_MAKEFILE_DIR))
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

config: include/config.h

sectioninfo: GNUmakefile $(KERNEL_MAP) include/kernel/generated/sections.h

all: bin/$(KERNEL)

iso: bin/$(KERNEL).iso

bin/$(KERNEL).iso: bin/$(KERNEL) $(COMMON_DEPS) $(LIMINE_CFG)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,mkdir -p iso_root/boot/limine,MKDIR,iso_root/boot/limine)
	$(call CMD,1,cp bin/$(KERNEL) iso_root/boot/,COPY,iso_root/boot/$(KERNEL))
	$(call CMD,1,cp $(LIMINE_CFG) iso_root/boot/limine/,COPY,iso_root/boot/limine/$(LIMINE_CFG))
	$(call CMD,1,cp $(LIMINE_DATADIR)/*.sys iso_root/boot/limine/,GLOBCOPY,$(LIMINE_DATADIR)/*.sys --> iso_root/boot/limine/)
	$(call CMD,1,cp $(LIMINE_DATADIR)/*.bin iso_root/boot/limine/,GLOBCOPY,$(LIMINE_DATADIR)/*.bin --> iso_root/boot/limine/)
	$(call CMD,1,mkdir -p iso_root/EFI/BOOT,MKDIR,iso_root/EFI/BOOT)
	$(call CMD,1,cp $(LIMINE_DATADIR)/BOOTX64.EFI iso_root/EFI/BOOT/,COPY,iso_root/EFI/BOOT/BOOTX64.EFI)
	$(call CMD,1,cp $(LIMINE_DATADIR)/BOOTIA32.EFI iso_root/EFI/BOOT/,COPY,iso_root/EFI/BOOT/BOOTIA32.EFI)
	$(call CMD,1, 														\
		$(XORRISO) -as mkisofs -b boot/limine/limine-bios-cd.bin 		\
			-no-emul-boot -boot-load-size 4 -boot-info-table 			\
			--efi-boot boot/limine/limine-uefi-cd.bin 					\
			-efi-boot-part --efi-boot-image --protective-msdos-label 	\
			iso_root -o $@,XORRISO,$@									\
	)
	$(call CMD,1,$(LIMINE) bios-install $@,LIMINE,$@)

bin/$(KERNEL): $(COMMON_DEPS) $(OBJ) $(KERNEL_MAP)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KLD) $(KLDFLAGS) $(OBJ) -o $@,LD,$@)
#printf '\003' | dd of=$@ bs=1 count=1 seek=16 conv=notrunc 2>/dev/null
	$(if $(call not,$(DEBUG_KERNEL)), \
		$(call CMD,1,$(KSTRIP) --strip-debug $@,STRIP,$@) \
	)

# Include header dependencies.
-include $(HEADER_DEPS)

# Generate header dependencies.
obj/%.c.d: src/%.c GNUmakefile
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KPP) $(KCFLAGS) $(KCPPFLAGS) -M -MG -MP -MT $(@:.d=.o) $< -MF $@ > /dev/null,MKDEP,$<)
obj/boot/%.c.d: src/boot/%.c GNUmakefile
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KPP) $(BOOT_KCFLAGS) $(KCPPFLAGS) -M -MG -MP -MT $(@:.d=.o) $< -MF $@ > /dev/null,MKDEP,$<)

# Compilation rules for *.c files.
obj/%.c.o: src/%.c $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@,GCC,$<)
obj/boot/%.c.o: src/boot/%.c $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@,GCC,$<)

# Compilation rules for *.s files.
obj/%.s.o: src/%.s $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KAS) $(KASFLAGS) $< -o $@,GAS,$<)
obj/boot/%.s.o: src/boot/%.s $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KAS) $(BOOT_KASFLAGS) $< -o $@,GAS,$<)

# Compilation rules for *.S files.
obj/%.S.o: src/%.S $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KCC) $(KCFLAGS) $(KCPPFLAGS) -c $< -o $@,GCC,$<)
obj/boot/%.S.o: src/boot/%.S $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KCC) $(BOOT_KCFLAGS) $(KCPPFLAGS) -c $< -o $@,GCC,$<)

# Compilation rules for *.asm (nasm) files.
obj/%.asm.o: src/%.asm $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KNASM) $(KNASMFLAGS) $< -o $@,NASM,$<)
obj/boot/%.asm.o: src/boot/%.asm $(COMMON_DEPS)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(KNASM) $(KNASMFLAGS) $< -o $@,NASM,$<)

override define MK_LIBGRAPHEME_CONFIG :=
$(call CMD,10,/usr/bin/true,FIX,glue/libgrapheme/config.mk)
@echo '## Automatically generated. Do not edit. ##' > glue/libgrapheme/config.mk
@echo 'PREFIX    = $(abspath .)'                    >> glue/libgrapheme/config.mk
@echo 'INCPREFIX = $(abspath ./include)'            >> glue/libgrapheme/config.mk
@echo 'LIBPREFIX = $(abspath ./lib)'                >> glue/libgrapheme/config.mk
@echo 'MANPREFIX = /tmp/share/man'                  >> glue/libgrapheme/config.mk
@echo ''											>> glue/libgrapheme/config.mk
@echo 'ANAME     = libgrapheme.a'                   >> glue/libgrapheme/config.mk
@echo ''											>> glue/libgrapheme/config.mk
@echo 'BUILD_CPPFLAGS = -D_ISOC99_SOURCE'		    >> glue/libgrapheme/config.mk
@echo 'BUILD_CFLAGS   = -std=c99 -Os -Wall	\
	-Wextra -Wpedantic -Wno-overlength-strings'     >> glue/libgrapheme/config.mk
@echo 'BUILD_LDFLAGS  = -s'						    >> glue/libgrapheme/config.mk
@echo ''											>> glue/libgrapheme/config.mk
@echo 'CPPFLAGS = -D_ISOC99_SOURCE $(KCPPFLAGS)'	>> glue/libgrapheme/config.mk
@echo 'CFLAGS   = -Wall	-Wextra -Wpedantic	\
	-Wno-overlength-strings $(KCFLAGS)'				>> glue/libgrapheme/config.mk
@echo ''											>> glue/libgrapheme/config.mk
@echo 'CC	    = $(KCC)'							>> glue/libgrapheme/config.mk
@echo 'AR	    = $(KAR)'							>> glue/libgrapheme/config.mk
@echo 'RANLIB   = $(KRANLIB)'						>> glue/libgrapheme/config.mk
@echo 'BUILD_CC = cc'								>> glue/libgrapheme/config.mk
@echo 'SH	    = sh'								>> glue/libgrapheme/config.mk
@echo ''											>> glue/libgrapheme/config.mk
endef

lib/libgrapheme.a:
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,2,mkdir -p glue/libgrapheme, MKDIR,glue/libgrapheme)
	$(call CMD,1,cat third_party/libgrapheme/src/character.c | sed -Ee 's/#include <stdio.h>//g' > glue/libgrapheme/src/character.c,FIX,glue/libgrapheme/src/character.c)
	$(MK_LIBGRAPHEME_CONFIG)
	$(SHELL) -c 'GLUEFILES=$$(find glue/libgrapheme -type f);	\
		for f in $$GLUEFILES; do 								\
			TOREPLACE=$$(echo $$f | sed -Ee 					\
				"s/glue\//third_party\//g");					\
			cp -T $$f $$TOREPLACE;								\
		done'
	$(call CMD,1,$(MAKE) -C third_party/libgrapheme libgrapheme.a,BUILD,libgrapheme.a)
	$(call CMD,1,cp third_party/libgrapheme/libgrapheme.a $$(dirname $@),COPY,$@)
	$(call CMD,1,cp third_party/libgrapheme/grapheme.h include/,COPY,include/grapheme.h)

override define GENSECINFO =
$(call CMD,2,mkdir -p "$$(dirname include/kernel/generated/sections.h)",MKDIR,$$(dirname include/kernel/generated/sections.h))
$(call CMD,1,$(__GENSECINFO_IMPL),GENSEC,)
endef

override define __GENSECINFO_IMPL =
../scripts/gensectioninfo.py 						\
--output-cfile include/kernel/generated/sections.h 	\
--output-linkerfile $(KERNEL_MAP) 					\
--input-linkerfile $(KERNEL_MAP).template 			\
$(KERNEL_SECTIONS)
endef

$(KERNEL_MAP): GNUmakefile $(KERNEL_MAP).template ../scripts/gensectioninfo.py
	$(GENSECINFO)

include/kernel/generated/sections.h: GNUmakefile $(KERNEL_MAP).template ../scripts/gensectioninfo.py
	$(GENSECINFO)

override KCPU_FEATURES := $(shell cpuid_tool --flags | grep --invert-match -e '-')
$(eval $(call PRINTVAR_RULE,KCPU_FEATURES))

override define GENCONFIG =
$(call CMD,10,/usr/bin/true,GENCONFIG,$@)
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
@echo "#undef  zerOS_CONFIG_UNDER_QEMU" >> $@
@echo "#define zerOS_CONFIG_UNDER_QEMU $(if $(KQEMU),1,0)" >> $@
@echo "" >> $@
@echo "#undef  zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS" >> $@
@echo "#define zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS 64UL" >> $@
@echo "" >> $@
@cat  $(KERNEL_SUPDEF_CONFIG) >> $@
@echo "" >> $@
@echo "#endif" >> $@
endef

override define SUPDEF =
../scripts/supdef.py $< -o $@ -Iinclude/config.d/
endef

include/config.h: include/config.h.in GNUmakefile
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,$(SUPDEF),SUPDEF,$<)

include/config.h.in: GNUmakefile $(CONFIG_DEPS)  $(KERNEL_SUPDEF_CONFIG)
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(GENCONFIG)

include/limine.h: $(KTOOLCHAIN_DIR)/include/limine.h
	$(call CMD,2,mkdir -p "$$(dirname $@)",MKDIR,$$(dirname $@))
	$(call CMD,1,cp $< $@,COPY,$@)

.PHONY: run
run: $(if $(DEBUG_KERNEL),run_debug,run_normal)

.PHONY: run_normal
run_normal: bin/$(KERNEL).iso
	$(call CMD,1,find -name debugcon.log -print -delete,DEL,debugcon.log)
	$(call CMD,1,$(KQEMU) $(KQEMU_RUNFLAGS) $<,QEMU,$<)

.PHONY: run_debug
run_debug: bin/$(KERNEL).iso
	$(call CMD,1,find -name debugcon.log -print -delete,DEL,debugcon.log)
	$(call START_DBG_SESSION,$<)

# Remove object files and the final executable.
.PHONY: clean
clean:
	$(call CMD,1,rm -rf bin,RM,bin)
	$(call CMD,1,rm -rf obj,RM,obj)
	$(call CMD,1,rm -f include/kernel/generated/sections.h,RM,include/kernel/generated/sections.h)
	$(call CMD,1,rm -rf iso_root,RM,iso_root)
	$(call CMD,1,rm -f debugcon.log,RM,debugcon.log)
	$(call CMD,1,rm -f $(KERNEL_MAP),RM,$(KERNEL_MAP))
