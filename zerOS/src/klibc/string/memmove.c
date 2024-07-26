#include <klibc/string.h>

static inline void* memmove_naive(void* dest, const void* src, size_t n);

extern void* memmove(void* dest, const void* src, size_t n)
{
    return memmove_naive(dest, src, n);
}

static inline void* memmove_naive(void* dest, const void* src, size_t n)
{
    unsigned char* d = dest;
    const unsigned char* s = src;
    if (d < s)
        while (n--)
            *d++ = *s++;
    else
    {
        d += n;
        s += n;
        while (n--)
            *--d = *--s;
    }
    return dest;
}