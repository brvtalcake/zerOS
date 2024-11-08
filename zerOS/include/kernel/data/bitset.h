#ifndef zerOS_KERNEL_DATA_BITSET_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#include <klibc/preprocessor/empty.h>

#include <machine/path.h>
#include MK_MACHINE_PATH(fast_data_types.h)

#endif

#if !defined(zerOS_KERNEL_DATA_BITSET_H_INCLUDED) || defined(zerOS_NEED_BITSET_IMPLEMENTATION)

#undef  __bitset_underlying_type_set_vectorized
#ifndef zerOS_BITSET_UNDERLYING_TYPE
    #define zerOS_BITSET_UNDERLYING_TYPE zerOS_fast_uint_t
    #define __bitset_underlying_type_set_vectorized(...) zerOS_fast_uint_set_vectorized(__VA_ARGS__)
#elif defined(zerOS_BITSET_UNDERLYING_TYPE_SET_VECTORIZED_FN)
    #define __bitset_underlying_type_set_vectorized(...) zerOS_BITSET_UNDERLYING_TYPE_SET_VECTORIZED_FN(__VA_ARGS__)
#else
    #define __bitset_underlying_type_set_vectorized(...)
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

static inline void zerOS_bitset_set(bitset_t bitset, const size_t bit)
{
    const size_t underlying_bits = sizeof(zerOS_BITSET_UNDERLYING_TYPE) * __CHAR_BIT__;

    const size_t index  = bit / underlying_bits;
    const size_t offset = bit % underlying_bits;

    bitset[index] |= (zerOS_BITSET_UNDERLYING_TYPE)1 << offset;
}

static inline void zerOS_bitset_clear(bitset_t bitset, const size_t bit)
{
    const size_t underlying_bits = sizeof(zerOS_BITSET_UNDERLYING_TYPE) * __CHAR_BIT__;

    const size_t index  = bit / underlying_bits;
    const size_t offset = bit % underlying_bits;

    bitset[index] &= ~((zerOS_BITSET_UNDERLYING_TYPE)1 << offset);
}

static inline bool zerOS_bitset_test(bitset_t bitset, const size_t bit)
{
    const size_t underlying_bits = sizeof(zerOS_BITSET_UNDERLYING_TYPE) * __CHAR_BIT__;

    const size_t index  = bit / underlying_bits;
    const size_t offset = bit % underlying_bits;

    return bitset[index] & ((zerOS_BITSET_UNDERLYING_TYPE)1 << offset);
}

/**
 * @brief Set all bits in the bitset.
 * @param bitset The bitset.
 * @param size Number of bits in the bitset (1-based maximum bit index).
 */
static inline void zerOS_bitset_set_all(bitset_t bitset, const size_t size)
{
    const size_t underlying_bits = sizeof(zerOS_BITSET_UNDERLYING_TYPE) * __CHAR_BIT__;
    // Round up, to fill the last element completely, even though it may not be fully used
    const size_t realsize = (size + underlying_bits - 1) / underlying_bits;
    const zerOS_BITSET_UNDERLYING_TYPE value = (zerOS_BITSET_UNDERLYING_TYPE)-1;

#if !(ISEMPTY(__bitset_underlying_type_set_vectorized(_x, _x, _x))) && \
    defined(__bitset_underlying_type_set_vectorized)
    __bitset_underlying_type_set_vectorized(
        bitset, realsize, value
    );
#else
    for (size_t i = 0; i < realsize; i++)
        bitset[i] = value;
#endif
}

/**
 * @brief Clear all bits in the bitset.
 * @param bitset The bitset.
 * @param size Number of bits in the bitset (1-based maximum bit index).
 */
static inline void zerOS_bitset_clear_all(bitset_t bitset, const size_t size)
{
    const size_t underlying_bits = sizeof(zerOS_BITSET_UNDERLYING_TYPE) * __CHAR_BIT__;
    // Round up, to fill the last element completely, even though it may not be fully used
    const size_t realsize = (size + underlying_bits - 1) / underlying_bits;
    const zerOS_BITSET_UNDERLYING_TYPE value = (zerOS_BITSET_UNDERLYING_TYPE)0;

#if !(ISEMPTY(__bitset_underlying_type_set_vectorized(_x, _x, _x))) && \
    defined(__bitset_underlying_type_set_vectorized)
    __bitset_underlying_type_set_vectorized(
        bitset, realsize, value
    );
#else
    for (size_t i = 0; i < realsize; i++)
        bitset[i] = value;
#endif
}

#endif

#ifndef zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#define zerOS_KERNEL_DATA_BITSET_H_INCLUDED
#endif