
#ifdef __has_include_next
#  if __has_include_next(<stddef.h>)
#    include_next <stddef.h>
#  else
#    error "Cannot include the next (standard) <stddef.h> header."
#  endif
#else
#  error    "Cannot include the next (standard) <stddef.h> header."
#endif

#ifndef zerOS_STDDEF_H_INCLUDED
#define zerOS_STDDEF_H_INCLUDED

#undef  nullptr
/**
 * @def nullptr
 * @brief The C23 null pointer constant.
 */
#ifndef __INTELLISENSE__
    #define nullptr nullptr
#else
    #define nullptr NULL
#endif

#undef  constexpr
/**
 * @def constexpr
 * @brief C23 constexpr keyword. Marks a variable as being a constant expression.
 */
#ifndef __INTELLISENSE__
    #define constexpr constexpr
#else
    #define constexpr
#endif

#endif
