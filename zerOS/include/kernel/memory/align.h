#ifndef zerOS_KERNEL_MEMORY_ALIGN_H_INCLUDED
#define zerOS_KERNEL_MEMORY_ALIGN_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdalign.h>

#include <misc/unique_ident.h>

static inline bool zerOS_is_pow_two(uintptr_t x)
{
    if (unlikely(x == 0))
        return false;
    return (x & (x - 1)) == 0;
}

static inline uintptr_t zerOS_align_up(uintptr_t x, size_t align)
{
    if (likely(zerOS_is_pow_two(align)))
        return (x + (align - 1)) & ~(align - 1);
    else
        return x + ((align - (x % align)) % align);
}

static inline uintptr_t zerOS_align_down(uintptr_t x, size_t align)
{
    if (likely(zerOS_is_pow_two(align)))
        return x & ~(align - 1);
    else
        return x - (x % align);
}

static inline bool zerOS_is_aligned(uintptr_t x, size_t align)
{
    if (likely(zerOS_is_pow_two(align)))
        return (x & (align - 1)) == 0;
    else
        return x % align == 0;
}

#endif
