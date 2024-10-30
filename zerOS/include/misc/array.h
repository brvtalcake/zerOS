#ifndef zerOS_MISC_ARRAY_H_INCLUDED
#define zerOS_MISC_ARRAY_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#undef  ALEN
/**
 * @def ALEN(array)
 * @brief Get the length of an array.
 * @param array The array to get the length of.
 */
#define ALEN(array) (sizeof(array) / sizeof((array)[0]))

#endif
