#include <klibc/string.h>
#include <klibc/detail/mem.h>
#include <stdint.h>

extern
int memcmp(const void* p1, const void* p2, size_t n)
{
    uintptr_t align_p1 = (uintptr_t)p1;
    uintptr_t align_p2 = (uintptr_t)p2;

    uintptr_t up_p1 = KLIBC_ALIGN_UP(align_p1, sizeof(uintptr_t));
}