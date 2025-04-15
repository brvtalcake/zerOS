#include <klibc/string.h>

static inline void* memset_naive(void* restrict dest, int c, size_t n);

extern void* memset(void* restrict dest, int c, size_t n)
{
    return memset_naive(dest, c, n);
}

static inline void* memset_naive(void* restrict dest, int c, size_t n)
{
    unsigned char* p = dest;
    while (n--)
        *p++ = c;
    return dest;
}