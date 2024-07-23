#include <klibc/string.h>

extern
void* memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    return __builtin_memcpy(dest, src, n);
}