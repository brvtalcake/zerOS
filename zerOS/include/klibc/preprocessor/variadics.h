#ifndef zerOS_KLIBC_PREPROCESSOR_VARIADICS_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_VARIADICS_H_INCLUDED

#include <config.h>

#include <chaos/preprocessor/tuple/eat.h>
#include <chaos/preprocessor/recursion/expr.h>
#include <chaos/preprocessor/recursion/basic.h>
#include <chaos/preprocessor/arithmetic/inc.h>
#include <chaos/preprocessor/control/if.h>

#include <klibc/preprocessor/expand.h>

#include <pp_empty/pp_is_empty.h>

#undef  KLIBC_PP_VA_COUNT
/**
 * @def KLIBC_PP_VA_COUNT(...)
 * @brief Count the number of arguments in a variadic macro.
 * @param ... The arguments.
 * @return The number of arguments.
 */
#define KLIBC_PP_VA_COUNT(...)      \
    KLIBC_PP_EXPAND(                \
        __KLIBC_PP_VA_COUNT_IMPL(   \
            0,                      \
            __VA_ARGS__             \
        )                           \
    )

#define __KLIBC_PP_VA_COUNT_RET(count, first) CHAOS_PP_IF(ISEMPTY(first))(count, CHAOS_PP_INC(count)) CHAOS_PP_EAT
#define __KLIBC_PP_VA_COUNT_IMPL_ID() __KLIBC_PP_VA_COUNT_IMPL
#define __KLIBC_PP_VA_COUNT_IMPL(count, first, ...) \
    CHAOS_PP_IF(ISEMPTY(__VA_ARGS__))               \
    (                                               \
        __KLIBC_PP_VA_COUNT_RET(count, first),      \
        CHAOS_PP_OBSTRUCT(                          \
            __KLIBC_PP_VA_COUNT_IMPL_ID             \
        )()                                         \
    )(                                              \
        CHAOS_PP_INC(count),                        \
        __VA_ARGS__                                 \
    )

static_assert(
    KLIBC_PP_VA_COUNT() == 0
);
static_assert(
    KLIBC_PP_VA_COUNT(one) == 1
);
static_assert(
    KLIBC_PP_VA_COUNT(one, two) == 2
);
static_assert(
    KLIBC_PP_VA_COUNT(BLAH1, BLAH2, BLAH3, BLAH4, BLAH5, BLAH6) == 6
);
static_assert(
    KLIBC_PP_VA_COUNT(BLAH1, BLAH2, BLAH3, BLAH4, BLAH5, BLAH6, BLAH7) == 7
);

#pragma push_macro("TEST")
#pragma push_macro("TEST_IMPL")

#undef  TEST
#undef  TEST_IMPL
#define TEST TEST_IMPL
#define TEST_IMPL we, are, testing, our, variadic, macro

static_assert(
    KLIBC_PP_VA_COUNT(some, more, args, here, TEST) == 6 + 4
);

#pragma pop_macro("TEST")
#pragma pop_macro("TEST_IMPL")

#endif
