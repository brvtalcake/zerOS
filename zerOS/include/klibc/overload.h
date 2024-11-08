#ifndef zerOS_KLIBC_OVERLOAD_H_INCLUDED
#define zerOS_KLIBC_OVERLOAD_H_INCLUDED

#include <chaos/preprocessor/arithmetic/sub.h>
#include <chaos/preprocessor/control/if.h>
#include <chaos/preprocessor/recursion/expr.h>

#include <klibc/detail/overload.h>
#include <klibc/preprocessor/empty.h>
#include <klibc/preprocessor/variadics.h>

// clang-format off
#undef  KLIBC_OVERLOAD_FN
/**
 * @def KLIBC_OVERLOAD_FN
 * @brief Overload a function with multiple signatures.
 * @details Use like this:
 * @code
 * #define my_function(...) KLIBC_OVERLOAD_FN(count, (my_function_overload1, ((float), (unsigned, int), (signed, char, ptr))), [other overloads...], __VA_ARGS__)
 * @endcode
 */
#define KLIBC_OVERLOAD_FN(count, ...) __KLIBC_OVERLOAD_FN(count, __VA_ARGS__)

#undef  __KLIBC_OVERLOAD_FN
#define __KLIBC_OVERLOAD_FN(count, ...)                     \
    __KLIBC_OVERLOAD_FN_IMPL(                               \
        __KLIBC_OVERLOAD_FN_FILTER_BY_PROVIDED_ARGCOUNT(    \
            CHAOS_PP_SUB(                                   \
                KLIBC_PP_VA_COUNT(__VA_ARGS__), count       \
            ),                                              \
            KLIBC_PP_VA_FROM_TO(0, count, __VA_ARGS__)      \
        )                                                   \
    )
// clang-format on
#endif
