override KCFLAGS += -Wall -Wextra -std=gnu23		\
	-xc -ffreestanding -fno-stack-protector			\
    -fno-stack-check -m64 -mno-mmx					\
	$(call CC_TUNE_FOR,$(KCPU))	-mno-sse -mno-sse2 	\
	-mno-red-zone -mno-avx -mno-avx2 -mno-avx512f	\
	-nodefaultlibs -nostdlib -nostartfiles 			\
	-m128bit-long-double -fno-lto -msoft-float

override KCXXFLAGS += -Wall -Wextra -std=gnu++23	\
	-xc++ -ffreestanding -fno-stack-protector		\
    -fno-stack-check -m64 -mno-mmx					\
	$(call CC_TUNE_FOR,$(KCPU))	-mno-sse -mno-sse2 	\
	-mno-red-zone -mno-avx -mno-avx2 -mno-avx512f	\
	-nodefaultlibs -nostdlib -nostartfiles 			\
	-m128bit-long-double -fno-lto -msoft-float		\
	-fno-rtti

override KCPPFLAGS += 			\
	-DCHAOS_PP_VARIADICS=1		\
	-Iinclude 		  			\
	-Ithird_party	  			\
	-Ithird_party/chaos-pp		\
	-Ithird_party/incbin		\
	-Ithird_party/stb			\
	-ftrack-macro-expansion=0

# Internal linker flags that should not be changed by the user.
override KLDFLAGS += -m elf_x86_64 -nostdlib -z text	\
	-z max-page-size=0x1000 -T $(KERNEL_MAP)

override KLINKFLAGS = -T $(KERNEL_MAP) -nostdlib -Wl,-z,text	\
	-Wl,-z,max-page-size=0x1000

override KNASMFLAGS += -Wall -f elf64

ifeq ($(call CC_SUPPORTS_OPTION,-Wno-deprecated),$(true))
	override KCFLAGS   += -Wno-deprecated
	override KCXXFLAGS += -Wno-deprecated
endif

ifeq ($(call CC_SUPPORTS_OPTION,-Wno-comment),$(true))
	override KCFLAGS   += -Wno-comment
	override KCXXFLAGS += -Wno-comment
endif

ifeq ($(call CC_SUPPORTS_OPTION,-mcmodel=kernel),$(true))
	override KCFLAGS   += -mcmodel=kernel
	override KCXXFLAGS += -mcmodel=kernel
else
	$(error Compiler does not support -mcmodel=kernel)
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fno-omit-frame-pointer),$(true))
#override BOOT_KCFLAGS += -fno-omit-frame-pointer
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fgraphite),$(true))
	override KCFLAGS   += -fgraphite
	override KCXXFLAGS += -fgraphite
endif

ifeq ($(call CC_SUPPORTS_OPTION,-fgraphite-identity),$(true))
	override KCFLAGS   += -fgraphite-identity
	override KCXXFLAGS += -fgraphite-identity
endif
