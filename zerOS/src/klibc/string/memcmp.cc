#include <klibc/string.h>
#include <stdint.h>

static inline int memcmp_naive(const void* p1, const void* p2, size_t n);

extern int memcmp(const void* p1, const void* p2, size_t n)
{
    return memcmp_naive(p1, p2, n);
}

static inline int memcmp_naive(const void* p1, const void* p2, size_t n)
{
    const unsigned char* s1 = p1;
    const unsigned char* s2 = p2;
    while (n--)
    {
        if (*s1 != *s2)
            return *s1 - *s2;
        s1++;
        s2++;
    }
    return 0;
}