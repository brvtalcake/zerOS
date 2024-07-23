#include <klibc/string.h>

extern
void* memmove(void* dest, const void* src, size_t n)
{
    return __builtin_memmove(dest, src, n);
}