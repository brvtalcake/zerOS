#include <klibc/string.h>
#include <misc/sections.h>
#include <misc/symbol.h>

static inline void* memcpy_boot(void* restrict dest, const void* restrict src, size_t n);

extern void* memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    if 
    return memcpy_boot(dest, src, n);
}

BOOT_FUNC
static inline void* memcpy_boot(void* restrict dest, const void* restrict src, size_t n)
{
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--)
        *d++ = *s++;
    return dest;
}

static inline void* memcpy_optimized(void* restrict dest, const void* restrict src, size_t n)
{
    return __builtin_memcpy(dest, src, n);
}