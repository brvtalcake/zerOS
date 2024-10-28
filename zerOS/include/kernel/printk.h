#ifndef zerOS_KERNEL_PRINTK_H_INCLUDED
#define zerOS_KERNEL_PRINTK_H_INCLUDED

#include <stdarg.h>

#include <misc/sections.h>
#include <misc/printf.h>

#include <kernel/compiler/cast.h>

#include <chaos/preprocessor/cat.h>

#undef  EPRI_CAST
#define EPRI_CAST(fmt_spec, ...) CHAOS_PP_CAT(__EPRI_CAST_, fmt_spec)(__VA_ARGS__)

#undef  __EPRI_CAST_d
#undef  __EPRI_CAST_u
#undef  __EPRI_CAST_x
#undef  __EPRI_CAST_X
#undef  __EPRI_CAST_p
#undef  __EPRI_CAST_b
#undef  __EPRI_CAST_o
#undef  __EPRI_CAST_c
#undef  __EPRI_CAST_s

#define __EPRI_CAST_d(value) reinterpret_cast(int64_t,     (value))
#define __EPRI_CAST_u(value) reinterpret_cast(uint64_t,    (value))
#define __EPRI_CAST_x(value) reinterpret_cast(uint64_t,    (value))
#define __EPRI_CAST_X(value) reinterpret_cast(uint64_t,    (value))
#define __EPRI_CAST_p(value) reinterpret_cast(uintptr_t,   (value))
#define __EPRI_CAST_b(value) reinterpret_cast(uint64_t,    (value))
#define __EPRI_CAST_o(value) reinterpret_cast(uint64_t,    (value))
#define __EPRI_CAST_c(value) reinterpret_cast(int,         (value))
#define __EPRI_CAST_s(value) reinterpret_cast(const char*, (value))

BOOT_FUNC
extern int zerOS_early_vprintk(const char* str, va_list varargs);

BOOT_FUNC
extern int zerOS_early_printk(const char* str, ...);

#endif
