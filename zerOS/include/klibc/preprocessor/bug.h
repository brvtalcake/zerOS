#ifndef zerOS_KLIBC_PREPROCESSOR_BUG_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_BUG_H_INCLUDED

#include <chaos/preprocessor/control/unless.h>
#include <chaos/preprocessor/control/when.h>
#include <chaos/preprocessor/logical/not.h>
#include <chaos/preprocessor/stringize.h>

#include <klibc/preprocessor/default_arg.h>

// clang-format off
#undef  __KLIBC_PP_BUILD_BUG_PRAGMA_ERROR_MSG
#define __KLIBC_PP_BUILD_BUG_PRAGMA_ERROR_MSG(...) \
    _Pragma(CHAOS_PP_STRINGIZE(GCC error __VA_ARGS__))

#undef  __KLIBC_PP_BUILD_BUG_UNLESS
#define __KLIBC_PP_BUILD_BUG_UNLESS(condstr, cond, ...) \
    CHAOS_PP_UNLESS(cond)(                               \
        __KLIBC_PP_BUILD_BUG_PRAGMA_ERROR_MSG(           \
            "Compile-time assertion failed: "            \
            KLIBC_PP_DEFAULT_ARG(                        \
                condstr,                                 \
                __VA_ARGS__                              \
            )                                            \
        )                                                \
    )

#undef  __KLIBC_PP_BUILD_BUG_WHEN
#define __KLIBC_PP_BUILD_BUG_WHEN(condstr, cond, ...)   \
    CHAOS_PP_WHEN(cond)(                                \
        __KLIBC_PP_BUILD_BUG_PRAGMA_ERROR_MSG(          \
            "Compile-time assertion failed: "           \
            KLIBC_PP_DEFAULT_ARG(                       \
                condstr,                                \
                __VA_ARGS__                             \
            )                                           \
        )                                               \
    )
                

#undef  KLIBC_PP_BUILD_BUG_WHEN
/**
 * @def KLIBC_PP_BUILD_BUG_WHEN(cond, ...)
 * @brief Build-time assertion.
 * @param cond The condition.
 * @param ...  The message, if needed.
 */
#define KLIBC_PP_BUILD_BUG_WHEN(cond, ...) \
    __KLIBC_PP_BUILD_BUG_WHEN(#cond, cond, __VA_ARGS__)

#undef  KLIBC_PP_BUILD_BUG_UNLESS
/**
 * @def KLIBC_PP_BUILD_BUG_UNLESS(cond, ...)
 * @brief Build-time assertion.
 * @param cond The condition.
 * @param ...  The message, if needed.
 */
#define KLIBC_PP_BUILD_BUG_UNLESS(cond, ...) \
    __KLIBC_PP_BUILD_BUG_UNLESS(#cond, cond, __VA_ARGS__)

// clang-format on

#endif
