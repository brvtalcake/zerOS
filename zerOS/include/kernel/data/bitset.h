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

static_assert(
    (zerOS_BITSET_UNDERLYING_TYPE)-1 > 0,
    "zerOS: zerOS_BITSET_UNDERLYING_TYPE must be an unsigned integer type"
);

// Define bitset_t as a pointer to the underlying type
typedef zerOS_BITSET_UNDERLYING_TYPE* bitset_t;

// TODO (fixme): All bitset operation are dependent on the underlying type being used
// TODO: Make `set_all` and `clear_all` functions accept the size of bits instead of the size of zerOS_BITSET_UNDERLYING_TYPE
//    composing the bitset

static inline void zerOS_bitset_set(bitset_t bitset, size_t bit)
{
    size_t index  = bit / zerOS_fast_uint_bits;
    size_t offset = bit % zerOS_fast_uint_bits;
    bitset[index] |= (zerOS_BITSET_UNDERLYING_TYPE)1 << offset;
}

static inline void zerOS_bitset_clear(bitset_t bitset, size_t bit)
{
    size_t index  = bit / zerOS_fast_uint_bits;
    size_t offset = bit % zerOS_fast_uint_bits;
    bitset[index] &= ~((zerOS_BITSET_UNDERLYING_TYPE)1 << offset);
}

static inline bool zerOS_bitset_test(bitset_t bitset, size_t bit)
{
    size_t index  = bit / zerOS_fast_uint_bits;
    size_t offset = bit % zerOS_fast_uint_bits;
    return bitset[index] & ((zerOS_BITSET_UNDERLYING_TYPE)1 << offset);
}

static inline void zerOS_bitset_set_all(bitset_t bitset, size_t size)
{
    zerOS_fast_uint_set_vectorized(bitset, size, (zerOS_BITSET_UNDERLYING_TYPE)-1);
}

static inline void zerOS_bitset_clear_all(bitset_t bitset, size_t size)
{
    zerOS_fast_uint_set_vectorized(bitset, size, (zerOS_BITSET_UNDERLYING_TYPE)0);
}

#endif

#ifndef zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#define zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#endif