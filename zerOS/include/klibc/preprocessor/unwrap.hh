#ifndef zerOS_KLIBC_PREPROCESSOR_UNWRAP_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_UNWRAP_H_INCLUDED

#include <chaos/preprocessor/control/variadic_if.h>
#include <chaos/preprocessor/logical/or.h>
#include <chaos/preprocessor/logical/and.h>
#include <chaos/preprocessor/tuple/rem.h>
#include <chaos/preprocessor/detection/is_variadic.h>
#include <chaos/preprocessor/debug/failure.h>

#include <klibc/preprocessor/empty.h>

#undef  __KLIBC_PP_UNWRAP_PARENTHESES
#undef  __KLIBC_PP_UNWRAP_CLASSIC
#undef  __KLIBC_PP_CHOOSE_UNWRAP_MACRO
#undef  __KLIBC_PP_UNWRAP_EMPTY

#define __KLIBC_PP_UNWRAP_PARENTHESES(...) CHAOS_PP_REM __VA_ARGS__
#define __KLIBC_PP_UNWRAP_CLASSIC(...)     __VA_ARGS__
#define __KLIBC_PP_CHOOSE_UNWRAP_MACRO(...)                     \
    CHAOS_PP_VARIADIC_IF(                                       \
        CHAOS_PP_IS_VARIADIC(__VA_ARGS__)                       \
    )(__KLIBC_PP_UNWRAP_PARENTHESES)(__KLIBC_PP_UNWRAP_CLASSIC)
#define __KLIBC_PP_UNWRAP_EMPTY(...)    \
    CHAOS_PP_OR                         \
    (                                   \
        ISEMPTY(__VA_ARGS__)            \
    )(                                  \
        CHAOS_PP_AND                    \
        (                               \
            CHAOS_PP_IS_VARIADIC(       \
                __VA_ARGS__             \
            )                           \
        )(                              \
            ISEMPTY(                    \
                CHAOS_PP_REM            \
                    __VA_ARGS__         \
            )                           \
        )                               \
    )


#undef  KLIBC_PP_UNWRAP
/**
 * @def KLIBC_PP_UNWRAP
 * @brief Unwrap a value or use a default value.
 * @param ... The value to unwrap.
 */
#define KLIBC_PP_UNWRAP(...)                        \
    CHAOS_PP_VARIADIC_IF(                           \
        __KLIBC_PP_UNWRAP_EMPTY(__VA_ARGS__)        \
    )(                                              \
        CHAOS_PP_FAILURE()                          \
    )(                                              \
        __KLIBC_PP_CHOOSE_UNWRAP_MACRO(__VA_ARGS__) \
    )(__VA_ARGS__)

#undef  KLIBC_PP_UNWRAP_OR
/**
 * @def KLIBC_PP_UNWRAP_OR
 * @brief Unwrap a value or use a default value.
 * @param default The default value (fails if the value is empty).
 * @param ...     The value to unwrap.
 */
#define KLIBC_PP_UNWRAP_OR(default, ...)            \
    CHAOS_PP_VARIADIC_IF(                           \
        __KLIBC_PP_UNWRAP_EMPTY(__VA_ARGS__)        \
    )(                                              \
        KLIBC_PP_UNWRAP(default)                    \
    )(                                              \
        KLIBC_PP_UNWRAP(__VA_ARGS__)                \
    )

#endif
