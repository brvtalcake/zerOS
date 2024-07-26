#ifndef zerOS_KLIBC_BITSET_H_INCLUDED
#define zerOS_KLIBC_BITSET_H_INCLUDED

#include <stdint.h>
#include <stdbool.h>
#include <klibc/string.h>

#undef  KLIBC_BITSET_T
/**
 * @def KLIBC_BITSET_T(size)
 * @brief The type of a bitset.
 * @param size The size of the bitset (in bits).
 */
#define KLIBC_BITSET_T(size) typeof(struct { uint64_t bits[((size) + 63) / 64]; })

#undef  KLIBC_BITSET_STATIC_INIT
/**
 * @def KLIBC_BITSET_STATIC_INIT
 * @brief The static initializer for a bitset.
 */
#define KLIBC_BITSET_STATIC_INIT { { 0 } }

#undef  KLIBC_BITSET_INIT
/**
 * @def KLIBC_BITSET_INIT(bitsetaddr)
 * @brief Initialize a bitset.
 * @param bitsetaddr The address of the bitset to initialize.
 */
#define KLIBC_BITSET_INIT(bitsetaddr) do { memset((bitsetaddr), 0, sizeof(*(bitsetaddr))); } while (false)

#undef  KLIBC_BITSET_SET
/**
 * @def KLIBC_BITSET_SET(bitsetaddr, bit)
 * @brief Set a bit in a bitset.
 * @param bitsetaddr The address of the bitset.
 * @param bit        The bit to set.
 */
#define KLIBC_BITSET_SET(bitsetaddr, bit) do { (bitsetaddr)->bits[(bit) / 64] |=  (1ULL << ((bit) % 64)); } while (false)

#undef  KLIBC_BITSET_CLEAR
/**
 * @def KLIBC_BITSET_CLEAR(bitsetaddr, bit)
 * @brief Clear a bit in a bitset.
 * @param bitsetaddr The address of the bitset.
 * @param bit        The bit to clear.
 */
#define KLIBC_BITSET_CLEAR(bitsetaddr, bit) do { (bitsetaddr)->bits[(bit) / 64] &= ~(1ULL << ((bit) % 64)); } while (false)

#undef  KLIBC_BITSET_TEST
/**
 * @def KLIBC_BITSET_TEST(bitsetaddr, bit)
 * @brief Test a bit in a bitset.
 * @param bitsetaddr The address of the bitset.
 * @param bit        The bit to test.
 */
#define KLIBC_BITSET_TEST(bitsetaddr, bit) ((bitsetaddr)->bits[(bit) / 64] & (1ULL << ((bit) % 64)))

#undef  KLIBC_BITSET_FIND_FIRST_SET
/**
 * @def KLIBC_BITSET_FIND_FIRST_SET(bitsetaddr, bit)
 * @brief Find the first set bit in a bitset.
 * @param bitsetaddr The address of the bitset.
 * @param bit        The bit to test.
 */
#define KLIBC_BITSET_FIND_FIRST_SET(bitsetaddr, bit) do {               \
    for ((bit) = 0; (bit) < sizeof((bitsetaddr)->bits) * 8; (bit)++)    \
        if (KLIBC_BITSET_TEST(bitsetaddr, bit))                         \
            break;                                                      \
} while (false)

#endif
