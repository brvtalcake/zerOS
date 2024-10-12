#ifndef zerOS_KERNEL_PRINTK_H_INCLUDED
#define zerOS_KERNEL_PRINTK_H_INCLUDED

#include <stdarg.h>

#include <misc/sections.h>

BOOT_FUNC
extern int zerOS_early_printk(const char* str, ...);

#endif
