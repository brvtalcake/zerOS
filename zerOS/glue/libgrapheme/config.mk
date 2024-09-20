## Automatically generated. Do not edit. ##
PREFIX    = /home/axel/Documents/programmation/osdev/zerOS/zerOS
INCPREFIX = /home/axel/Documents/programmation/osdev/zerOS/zerOS/include
LIBPREFIX = /home/axel/Documents/programmation/osdev/zerOS/zerOS/lib
MANPREFIX = /tmp/share/man

ANAME     = libgrapheme.a

BUILD_CPPFLAGS = -D_ISOC99_SOURCE
BUILD_CFLAGS   = -std=c99 -Os -Wall -Wextra -Wpedantic -Wno-overlength-strings
BUILD_LDFLAGS  = -s

CPPFLAGS = -D_ISOC99_SOURCE -DCHAOS_PP_VARIADICS=1 -Iinclude -Ithird_party -Ithird_party/chaos-pp -Ithird_party/incbin -Ithird_party/stb -ftrack-macro-expansion=0 -DCHAOS_PP_VARIADICS=1 -ftrack-macro-expansion=0
CFLAGS   = -Wall	-Wextra -Wpedantic -Wno-overlength-strings -g -O3 -pipe -mno-80387 -ftrack-macro-expansion=0 -Wall -Wextra -std=gnu23 -ffreestanding -fno-stack-protector -fno-stack-check -m64 -mno-red-zone 	-march=alderlake -mtune=alderlake	-nodefaultlibs -nostdlib -nostartfiles	-fno-lto -m128bit-long-double -Wno-deprecated -Wno-comment -mcmodel=kernel -fgraphite -fgraphite-identity

CC	    = /home/axel/Documents/programmation/osdev/zerOS/toolchain/install/bin/x86_64-elf-gcc
AR	    = /home/axel/Documents/programmation/osdev/zerOS/toolchain/install/bin/x86_64-elf-gcc-ar
RANLIB   = /home/axel/Documents/programmation/osdev/zerOS/toolchain/install/bin/x86_64-elf-gcc-ranlib
BUILD_CC = cc
SH	    = sh

