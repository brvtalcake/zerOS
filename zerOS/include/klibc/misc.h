#ifndef zerOS_KLIBC_MISC_H_INCLUDED
#define zerOS_KLIBC_MISC_H_INCLUDED

#include <stdbool.h>

#undef  KLIBC_TYPES_EQUAL
/**
 * @def KLIBC_TYPES_EQUAL(x, y)
 * @brief Check if two types or variable types are equal.
 * @param x The first type or variable.
 * @param y The second type or variable.
 * @return true if the types are equal, false otherwise.
 */
#define KLIBC_TYPES_EQUAL(x, y)         \
    ((bool)(                            \
        !!__builtin_types_compatible_p( \
            typeof((x)),                \
            typeof((y))                 \
        ) &&                            \
        !!(sizeof((x)) == sizeof((y)))  \
    ))

#endif
