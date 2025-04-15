#ifndef zerOS_KERNEL_COMPILER_CAST_H_INCLUDED
#define zerOS_KERNEL_COMPILER_CAST_H_INCLUDED

#ifndef __cplusplus

#include <misc/unique_ident.h>

#undef  reinterpret_cast
/**
 * @def reinterpret_cast(type, value)
 * @brief Casts a value to a different type.
 * @param type The type to cast to.
 * @param value The value to cast.
 * @return The casted value.
 */
#define reinterpret_cast(type, value)   \
    ({                                  \
        union                           \
        {                               \
            typeof_unqual(value)        \
                UNIQUE(from);           \
            type UNIQUE(to);            \
        } UNIQUE(cast) = {              \
            .UNIQUE(from) = value       \
        };                              \
        UNIQUE(cast).UNIQUE(to);        \
    })

#endif

#endif
