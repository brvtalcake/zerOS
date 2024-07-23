override BOOT_KCFLAGS := $(KCFLAGS) -Wall -Wextra 	\
	-std=gnu23 -ffreestanding -fno-stack-protector 	\
    -fno-stack-check -fPIE -m64 -mno-80387 -mno-mmx	\
	$(call CC_TUNE_FOR,x86-64)	-mno-sse -mno-sse2 	\
	-mno-red-zone -mno-avx -mno-avx2 -mno-avx512f	\
	-nodefaultlibs -nostdlib -nostartfiles 			\
	-m128bit-long-double

override KCFLAGS += -Wall -Wextra -std=gnu23	\
	-ffreestanding -fno-stack-protector 		\
    -fno-stack-check -fPIE -m64 -mno-red-zone	\
	$(call CC_TUNE_FOR,$(KCPU))	-nodefaultlibs 	\
	-nostdlib -nostartfiles						\
	-m128bit-long-double

override KCPPFLAGS := -I include $(KCPPFLAGS)

# Internal linker flags that should not be changed by the user.
override KLDFLAGS += -m elf_x86_64 -nostdlib -pie	\
    -z text -z max-page-size=0x1000 -T $(KERNEL_MAP)

override KNASMFLAGS += -Wall -f elf64