#ifndef zerOS_KLIBC_STRING_H_INCLUDED
#define zerOS_KLIBC_STRING_H_INCLUDED

#include <stddef.h>

/* TODO: For "naive" implementation, add some pragmas or attributes to disable isa extensions */

extern void* memset(void* restrict dest, int c, size_t n);
extern void* memcpy(void* restrict dest, const void* restrict src, size_t n);
extern void* memmove(void* dest, const void* src, size_t n);
extern int memcmp(const void* p1, const void* p2, size_t n);

#endif
