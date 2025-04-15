// clang-format off

#ifndef zerOS_KLIBC_PREPROCESSOR_VARIADICS_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_VARIADICS_H_INCLUDED

#include <config.h>

#include <chaos/preprocessor/arithmetic.h>
#include <chaos/preprocessor/comparison.h>
#include <chaos/preprocessor/control.h>
#include <chaos/preprocessor/debug.h>
#include <chaos/preprocessor/lambda.h>
#include <chaos/preprocessor/logical.h>
#include <chaos/preprocessor/recursion.h>
#include <chaos/preprocessor/repetition/enum.h>
#include <chaos/preprocessor/seq.h>
#include <chaos/preprocessor/tuple/eat.h>
#include <chaos/preprocessor/tuple/filter.h>
#include <chaos/preprocessor/tuple/transform.h>

#include <klibc/preprocessor/bug.h>
#include <klibc/preprocessor/empty.h>
#include <klibc/preprocessor/expand.h>
#include <klibc/preprocessor/seq.h>
#include <klibc/preprocessor/tuple.h>

#undef  KLIBC_PP_VARIADIC_TO_SEQ
/**
 * @def KLIBC_PP_VARIADIC_TO_SEQ(...)
 * @brief Convert variadic arguments to a sequence.
 * @param ... The arguments.
 * @return The sequence.
 */
#define KLIBC_PP_VARIADIC_TO_SEQ(...)       \
    KLIBC_PP_EXPAND(                        \
        __KLIBC_PP_VARIADIC_TO_SEQ_IMPL(    \
            __VA_ARGS__                     \
        )                                   \
    )

#undef  __KLIBC_PP_VARIADIC_TO_SEQ_IMPL_ID
#define __KLIBC_PP_VARIADIC_TO_SEQ_IMPL_ID() __KLIBC_PP_VARIADIC_TO_SEQ_IMPL

#undef  __KLIBC_PP_VARIADIC_TO_SEQ_IMPL
#define __KLIBC_PP_VARIADIC_TO_SEQ_IMPL(first, ...) \
    CHAOS_PP_IF(KLIBC_PP_IS_EMPTY(__VA_ARGS__))(    \
        CHAOS_PP_EAT,                               \
        (first)                                     \
        CHAOS_PP_OBSTRUCT(                          \
            __KLIBC_PP_VARIADIC_TO_SEQ_IMPL_ID      \
        )()                                         \
    )(                                              \
        __VA_ARGS__                                 \
    )


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

#undef  __KLIBC_PP_VA_COUNT_RET
#define __KLIBC_PP_VA_COUNT_RET(count, first) CHAOS_PP_IF(ISEMPTY(first))(count, CHAOS_PP_INC(count)) CHAOS_PP_EAT

#undef  __KLIBC_PP_VA_COUNT_IMPL_ID
#define __KLIBC_PP_VA_COUNT_IMPL_ID() __KLIBC_PP_VA_COUNT_IMPL

#undef  __KLIBC_PP_VA_COUNT_IMPL
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

#undef  KLIBC_PP_VA_FROM_TO
/**
 * @def KLIBC_PP_VA_FROM_TO(from, to, ...)
 * @brief Get the arguments from the range [from, to[.
 */
#define KLIBC_PP_VA_FROM_TO(from, to, ...)      \
    KLIBC_PP_EXPAND(                            \
        CHAOS_PP_EXPR(                          \
            KLIBC_PP_BUILD_BUG_UNLESS(          \
                CHAOS_PP_BITAND(                \
                    CHAOS_PP_LESS_EQUAL(        \
                        from, to                \
                    )                           \
                )(                              \
                    CHAOS_PP_LESS_EQUAL(        \
                        to, KLIBC_PP_VA_COUNT(  \
                            __VA_ARGS__         \
                        )                       \
                    )                           \
                ),                              \
                "KLIBC_PP_VA_FROM_TO: "         \
                "invalid range"                 \
            )                                   \
            KLIBC_PP_SEQ_TO_VARIADIC(           \
                __KLIBC_PP_VA_FROM_TO_IMPL(     \
                    from,                       \
                    to,                         \
                    __VA_ARGS__                 \
                )                               \
            )                                   \
        )                                       \
    )

#undef  __KLIBC_PP_VA_FROM_TO_MK_INDEX_HELPER
#define __KLIBC_PP_VA_FROM_TO_MK_INDEX_HELPER(_unused, used, ...) used

#undef  __KLIBC_PP_VA_FROM_TO_MK_INDEX
#define __KLIBC_PP_VA_FROM_TO_MK_INDEX(tuplesize)       \
    KLIBC_PP_VARIADIC_TO_SEQ(                           \
        CHAOS_PP_EXPR(                                  \
            CHAOS_PP_ENUM(                              \
                tuplesize,                              \
                __KLIBC_PP_VA_FROM_TO_MK_INDEX_HELPER   \
            )                                           \
        )                                               \
    )

#undef  __KLIBC_PP_VA_FROM_TO_IMPL
#define __KLIBC_PP_VA_FROM_TO_IMPL(from, to, ...)   \
    __KLIBC_PP_VA_FROM_TO_FILTER(                   \
        from,                                       \
        to,                                         \
        KLIBC_PP_SEQ_ZIP(                           \
            KLIBC_PP_VARIADIC_TO_SEQ(__VA_ARGS__),  \
            __KLIBC_PP_VA_FROM_TO_MK_INDEX(         \
                KLIBC_PP_VA_COUNT(__VA_ARGS__)      \
            )                                       \
        )                                           \
    )

#undef  __KLIBC_PP_VA_FROM_TO_FILTER_IMPL
#define __KLIBC_PP_VA_FROM_TO_FILTER_IMPL(_, elem, from, to, ...)   \
    CHAOS_PP_BITAND(                                                \
        CHAOS_PP_GREATER_EQUAL(                                     \
            KLIBC_PP_TUPLE_ELEM(1, elem),                           \
            from                                                    \
        )                                                           \
    )(                                                              \
        CHAOS_PP_LESS(                                              \
            KLIBC_PP_TUPLE_ELEM(1, elem),                           \
            to                                                      \
        )                                                           \
    )

#undef  __KLIBC_PP_VA_FROM_TO_FINAL_TRANSFORM
#define __KLIBC_PP_VA_FROM_TO_FINAL_TRANSFORM(_, elem) KLIBC_PP_TUPLE_ELEM(0, elem)

#undef  __KLIBC_PP_VA_FROM_TO_FILTER
#define __KLIBC_PP_VA_FROM_TO_FILTER(from, to, seq)     \
    CHAOS_PP_EXPR(                                      \
        CHAOS_PP_VARIADIC_SEQ_TRANSFORM(                \
            __KLIBC_PP_VA_FROM_TO_FINAL_TRANSFORM,      \
            CHAOS_PP_VARIADIC_SEQ_FILTER(               \
                __KLIBC_PP_VA_FROM_TO_FILTER_IMPL,      \
                seq, from, to                           \
            )                                           \
        )                                               \
    )

static_assert(KLIBC_PP_VA_COUNT() == 0);
static_assert(KLIBC_PP_VA_COUNT(one) == 1);
static_assert(KLIBC_PP_VA_COUNT(one, two) == 2);
static_assert(KLIBC_PP_VA_COUNT(BLAH1, BLAH2, BLAH3, BLAH4, BLAH5, BLAH6) == 6);
static_assert(KLIBC_PP_VA_COUNT(BLAH1, BLAH2, BLAH3, BLAH4, BLAH5, BLAH6, BLAH7) == 7);

#pragma push_macro("TEST")
#pragma push_macro("TEST_IMPL")

#undef  TEST
#undef  TEST_IMPL

#define TEST      TEST_IMPL
#define TEST_IMPL we, are, testing, our, variadic, macro

static_assert(KLIBC_PP_VA_COUNT(some, more, args, here, TEST) == 6 + 4);

#pragma pop_macro("TEST")
#pragma pop_macro("TEST_IMPL")

#endif

// clang-format on
