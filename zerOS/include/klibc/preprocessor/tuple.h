#ifndef zerOS_KLIBC_PREPROCESSOR_TUPLE_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_TUPLE_H_INCLUDED

#include <chaos/preprocessor/tuple/rem.h>

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

#endif
