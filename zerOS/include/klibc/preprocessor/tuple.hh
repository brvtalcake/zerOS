#ifndef zerOS_KLIBC_PREPROCESSOR_TUPLE_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_TUPLE_H_INCLUDED

#include <chaos/preprocessor/comparison/equal.h>
#include <chaos/preprocessor/tuple/for_each_i.h>
#include <chaos/preprocessor/tuple/rem.h>
#include <chaos/preprocessor/tuple/size.h>
#include <map/map.h>

#include <klibc/preprocessor/bug.h>

// clang-format off

#undef  KLIBC_PP_TUPLE_ELEM
/**
 * @def KLIBC_PP_TUPLE_ELEM(i, tuple)
 * @brief Get the i-th element of a tuple.
 * @param i     The index.
 * @param tuple The tuple.
 * @return      The i-th element.
 */
#define KLIBC_PP_TUPLE_ELEM(i, tuple) CHAOS_PP_TUPLE_ELEM(~, i, tuple)

#undef  __KLIBC_PP_TUPLE_HEAD_IMPL
#define __KLIBC_PP_TUPLE_HEAD_IMPL(first, ...) first

#undef  __KLIBC_PP_TUPLE_TAIL_IMPL
#define __KLIBC_PP_TUPLE_TAIL_IMPL(first, ...) __VA_ARGS__

#undef  KLIBC_PP_TUPLE_HEAD
/**
 * @def KLIBC_PP_TUPLE_HEAD(tuple)
 * @brief Get the first element of a tuple.
 * @param tuple The tuple.
 * @return The first element.
 */
#define KLIBC_PP_TUPLE_HEAD(tuple) __KLIBC_PP_TUPLE_HEAD_IMPL(CHAOS_PP_REM_CTOR(tuple))

#undef  KLIBC_PP_TUPLE_TAIL
/**
 * @def KLIBC_PP_TUPLE_TAIL(tuple)
 * @brief Get the tail of a tuple.
 * @param tuple The tuple.
 * @return The tail.
 */
#define KLIBC_PP_TUPLE_TAIL(tuple) __KLIBC_PP_TUPLE_TAIL_IMPL(CHAOS_PP_REM_CTOR(tuple))

#undef  KLIBC_PP_TUPLE_ZIP
/**
 * @def KLIBC_PP_TUPLE_ZIP(tuple1, tuple2)
 * @brief Zips two tuples.
 * @param tuple1 The first tuple.
 * @param tuple2 The second tuple.
 * @return The zipped tuple.
 */
#define KLIBC_PP_TUPLE_ZIP(tuple1, tuple2) \
    CHAOS_PP_EXPR(__KLIBC_PP_TUPLE_ZIP_IMPL(tuple1, tuple2))

#undef  __KLIBC_PP_TUPLE_ZIP_IMPL
#define __KLIBC_PP_TUPLE_ZIP_IMPL(tuple1, tuple2)   \
    KLIBC_PP_BUILD_BUG_UNLESS(                      \
        CHAOS_PP_EQUAL(                             \
            CHAOS_PP_TUPLE_SIZE(tuple1),            \
            CHAOS_PP_TUPLE_SIZE(tuple2)             \
        ),                                          \
        "Tuples must have the same size."           \
    )                                               \
    (                                               \
        CHAOS_PP_TUPLE_FOR_EACH_I(                  \
            __KLIBC_PP_TUPLE_ZIP_OP,                \
            tuple1, tuple2                          \
        )                                           \
    )

#undef  __KLIBC_PP_TUPLE_ZIP_OP
#define __KLIBC_PP_TUPLE_ZIP_OP(s, i, elem, tuple) \
    (elem, KLIBC_PP_TUPLE_ELEM(i, tuple))

// clang-format on

#endif // 0
