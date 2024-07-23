#include <klibc/string.h>

extern
void* memset(void* restrict dest, int c, size_t n)
{
    return __builtin_memset(dest, c, n);
}