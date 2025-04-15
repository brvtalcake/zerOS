#ifndef zerOS_MISC_BITS_H_INCLUDED
#define zerOS_MISC_BITS_H_INCLUDED

#include <stdint.h>
#include <stddef.h>

#undef  GET_BITS_AT
/**
 * @def GET_BITS_AT
 * @brief Get the bits at a specific position.
 * @param value The value to extract the bits from.
 * @param start The starting bit.
 * @param end The ending bit.
 */
#define GET_BITS_AT(value, start, end) (((value) >> (start)) & (((typeof_unqual(value))1 << ((end) - (start) + 1)) - 1))

#undef  EVEN
/**
 * @def EVEN
 * @brief Check if a number is even.
 * @param n The number to check.
 */
#define EVEN(n) ((n) % 2 == 0)

#undef  ODD
/**
 * @def ODD
 * @brief Check if a number is odd.
 * @param n The number to check.
 */
#define ODD(n) ((n) % 2 != 0)

#endif
