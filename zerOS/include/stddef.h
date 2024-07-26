#ifndef zerOS_STDDEF_H_INCLUDED
#define zerOS_STDDEF_H_INCLUDED

#undef  nullptr
/**
 * @def nullptr
 * @brief The null pointer constant.
 */
#define nullptr nullptr

#undef  constexpr
/**
 * @def constexpr
 * @brief Marks a variable as being a constant expression.
 */
#define constexpr constexpr

#include_next <stddef.h>

#endif
