#ifndef zerOS_MISC_ARRAY_H_INCLUDED
#define zerOS_MISC_ARRAY_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#undef  ARRAY_LEN
/**
 * @def ARRAY_LEN
 * @brief Get the length of an array.
 * @param array The array to get the length of.
 */
#define ARRAY_LEN(array) (sizeof(array) / sizeof((array)[0]))

#endif
