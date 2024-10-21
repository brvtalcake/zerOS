#ifndef zerOS_KERNEL_DATA_BITSET_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#include <machine/path.h>
#include MK_MACHINE_PATH(fast_data_types.h)

#endif

#if !defined(zerOS_KERNEL_DATA_BITSET_H_INCLUDED) || defined(zerOS_NEED_BITSET_IMPLEMENTATION)

#ifndef zerOS_BITSET_UNDERLYING_TYPE
#define zerOS_BITSET_UNDERLYING_TYPE zerOS_fast_uint_t
#endif

// Define bitset_t as a pointer to the underlying type
typedef zerOS_BITSET_UNDERLYING_TYPE* bitset_t;

static inline void zerOS_bitset_set(bitset_t bitset, size_t bit)
{
    *bitset |= ((zerOS_BITSET_UNDERLYING_TYPE)1 << bit);
}

static inline void zerOS_bitset_clear(bitset_t bitset, size_t bit)
{
    *bitset &= ~((zerOS_BITSET_UNDERLYING_TYPE)1 << bit);
}

static inline bool zerOS_bitset_test(bitset_t bitset, size_t bit)
{
    return *bitset & ((zerOS_BITSET_UNDERLYING_TYPE)1 << bit);
}

#endif

#ifndef zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#define zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#endif