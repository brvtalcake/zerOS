#ifndef zerOS_KERNEL_PRINTK_H_INCLUDED
#define zerOS_KERNEL_PRINTK_H_INCLUDED

#include <stdarg.h>

#include <misc/sections.h>
#include <misc/printf.h>

BOOT_FUNC
extern int zerOS_early_vprintk(const char* str, va_list varargs);

BOOT_FUNC PRINTF_LIKE(1, 2)
extern int zerOS_early_printk(const char* str, ...);

#endif
