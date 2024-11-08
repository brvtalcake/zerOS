#ifndef zerOS_KLIBC_PREPROCESSOR_EMPTY_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_EMPTY_H_INCLUDED

#include <chaos/preprocessor/logical/not.h>

#undef  ___ARG16
#undef  __HAS_COMMA
#define ___ARG16(_0, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, ...) _15
#define __HAS_COMMA(...) \
    ___ARG16(__VA_ARGS__, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0)

#undef  KLIBC_PP_IS_EMPTY
/**
 * @def KLIBC_PP_IS_EMPTY(...)
 * @brief Tests if the arguments are empty.
 * @param ... The arguments.
 * @return 1 if the arguments are empty, 0 otherwise.
 */
#define KLIBC_PP_IS_EMPTY(...) __KLIBC_PP_IS_EMPTY_IMPL(__VA_ARGS__)

#undef  __KLIBC_PP_IS_EMPTY_IMPL
#define __KLIBC_PP_IS_EMPTY_IMPL(...) CHAOS_PP_NOT(__HAS_COMMA(__VA_OPT__(,)))

#undef  ISEMPTY
#define ISEMPTY(...) KLIBC_PP_IS_EMPTY(__VA_ARGS__)

/*
 * Examples:
 * #define EMPTY() EMPTY_
 * #define EMPTY_
 * 
 * ISEMPTY() // 1
 * ISEMPTY(,) // 0
 * ISEMPTY(EMPTY) // 0
 * ISEMPTY(EMPTY()) // 1
 */


#endif
