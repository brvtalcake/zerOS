override BOOT_KCFLAGS := $(KCFLAGS) -Wall -Wextra 	\
	-std=gnu23 -ffreestanding -fno-stack-protector 	\
    -fno-stack-check -m64 -mno-mmx					\
	$(call CC_TUNE_FOR,x86-64)	-mno-sse -mno-sse2 	\
	-mno-red-zone -mno-avx -mno-avx2 -mno-avx512f	\
	-nodefaultlibs -nostdlib -nostartfiles 			\
	-m128bit-long-double -fno-lto

override KCFLAGS += -Wall -Wextra -std=gnu23	\
	-ffreestanding -fno-stack-protector 		\
    -fno-stack-check -m64 -mno-red-zone			\
	$(call CC_TUNE_FOR,$(KCPU))	-nodefaultlibs 	\
	-nostdlib -nostartfiles	-fno-lto			\
	-m128bit-long-double

override KCPPFLAGS := 		\
	-Iinclude 		  		\
	-Ithird_party	  		\
	-Ithird_party/chaos-pp	\
	$(KCPPFLAGS)

# Internal linker flags that should not be changed by the user.
override KLDFLAGS += -m elf_x86_64 -nostdlib -z text	\
	-z max-page-size=0x1000 -T $(KERNEL_MAP)

override KLINKFLAGS = -T $(KERNEL_MAP) -nostdlib -Wl,-z,text	\
	-Wl,-z,max-page-size=0x1000

override KNASMFLAGS += -Wall -f elf64

ifeq ($(call CC_SUPPORTS_OPTION,-Wno-deprecated),$(true))
	override KCFLAGS += -Wno-deprecated
	override BOOT_KCFLAGS += -Wno-deprecated
endif

ifeq ($(call CC_SUPPORTS_OPTION,-Wno-comment),$(true))
	override KCFLAGS += -Wno-comment
	override BOOT_KCFLAGS += -Wno-comment
endif

ifeq ($(call CC_SUPPORTS_OPTION,-mcmodel=kernel),$(true))
	override KCFLAGS += -mcmodel=kernel
	override BOOT_KCFLAGS += -mcmodel=kernel
else
	$(error Compiler does not support -mcmodel=kernel)
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fno-omit-frame-pointer),$(true))
	override BOOT_KCFLAGS += -fno-omit-frame-pointer
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fgraphite),$(true))
	override KCFLAGS += -fgraphite
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fgraphite-identity),$(true))
	override KCFLAGS += -fgraphite-identity
endif
