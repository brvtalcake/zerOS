#include <klibc/string.h>
#include <misc/sections.h>

static inline void* memcpy_naive(void* restrict dest, const void* restrict src, size_t n);

extern void* memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    return memcpy_naive(dest, src, n);
}

static inline void* memcpy_naive(void* restrict dest, const void* restrict src, size_t n)
{
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--)
        *d++ = *s++;
    return dest;
}
