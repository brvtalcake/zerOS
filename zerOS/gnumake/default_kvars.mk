define DEFAULT_VAR =
	ifeq ($(origin $1),default)
		override $(1) = $(2)
	endif
	ifeq ($(origin $1),undefined)
		override $(1) = $(2)
	endif
endef

override DEFAULT_SHELL := /bin/bash
$(eval $(call DEFAULT_VAR,SHELL,$(DEFAULT_SHELL)))

override DEFAULT_VERBOSE := 0
$(eval $(call DEFAULT_VAR,VERBOSE,$(DEFAULT_VERBOSE)))

override DEFAULT_KARCH := x86_64
$(eval $(call DEFAULT_VAR,KARCH,$(DEFAULT_KARCH)))
override DEFAULT_KCPU := $(shell gcc -march=native -Q --help=target | awk -e '/^\s*-march=/ {print $$2; exit}')
$(eval $(call DEFAULT_VAR,KCPU,$(DEFAULT_KCPU)))

override DEFAULT_KTOOLCHAIN_DIR := $(realpath ../toolchain/install/)
$(eval $(call DEFAULT_VAR,KTOOLCHAIN_DIR,$(DEFAULT_KTOOLCHAIN_DIR)))

override DEFAULT_KNASM := $(KTOOLCHAIN_DIR)/bin/nasm
$(eval $(call DEFAULT_VAR,KNASM,$(DEFAULT_KNASM)))
override DEFAULT_KCC := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-gcc
$(eval $(call DEFAULT_VAR,KCC,$(DEFAULT_KCC)))
override DEFAULT_KLD := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-ld
$(eval $(call DEFAULT_VAR,KLD,$(DEFAULT_KLD)))
override DEFAULT_KAS := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-as
$(eval $(call DEFAULT_VAR,KAS,$(DEFAULT_KAS)))
override DEFAULT_KOBJCOPY := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-objcopy
$(eval $(call DEFAULT_VAR,KOBJCOPY,$(DEFAULT_KOBJCOPY)))
override DEFAULT_KOBJDUMP := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-objdump
$(eval $(call DEFAULT_VAR,KOBJDUMP,$(DEFAULT_KOBJDUMP)))
override DEFAULT_KSTRIP := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-strip
$(eval $(call DEFAULT_VAR,KSTRIP,$(DEFAULT_KSTRIP)))
override DEFAULT_KAR := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-gcc-ar
$(eval $(call DEFAULT_VAR,KAR,$(DEFAULT_KAR)))
override DEFAULT_KRANLIB := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-gcc-ranlib
$(eval $(call DEFAULT_VAR,KRANLIB,$(DEFAULT_KRANLIB)))
override DEFAULT_KNM := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-gcc-nm
$(eval $(call DEFAULT_VAR,KNM,$(DEFAULT_KNM)))
override DEFAULT_KSIZE := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-size
$(eval $(call DEFAULT_VAR,KSIZE,$(DEFAULT_KSIZE)))
override DEFAULT_KPP := $(KTOOLCHAIN_DIR)/bin/$(KARCH)-elf-cpp
$(eval $(call DEFAULT_VAR,KPP,$(DEFAULT_KPP)))

override DEFAULT_KCFLAGS := -g -O3 -pipe -mno-80387
$(eval $(call DEFAULT_VAR,KCFLAGS,$(DEFAULT_KCFLAGS)))
override DEFAULT_KCPPFLAGS := -DCHAOS_PP_VARIADICS=1
$(eval $(call DEFAULT_VAR,KCPPFLAGS,$(DEFAULT_KCPPFLAGS)))
override DEFAULT_KNASMFLAGS := -F dwarf -g
$(eval $(call DEFAULT_VAR,KNASMFLAGS,$(DEFAULT_KNASMFLAGS)))
override DEFAULT_KLDFLAGS :=
$(eval $(call DEFAULT_VAR,KLDFLAGS,$(DEFAULT_KLDFLAGS)))
override DEFAULT_KASFLAGS := # TODO
$(eval $(call DEFAULT_VAR,KASFLAGS,$(DEFAULT_KASFLAGS)))

LIMINE := $(KTOOLCHAIN_DIR)/bin/limine
LIMINE_CFG := limine.cfg
LIMINE_DATADIR := $(shell $(LIMINE) --print-datadir)

XORRISO := $(shell which xorriso)

DEBUG_KERNEL ?= $(false)