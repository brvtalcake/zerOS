#ifndef zerOS_KLIBC_PREPROCESSOR_BINARY_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_BINARY_H_INCLUDED

#include <klibc/preprocessor/for.h>
#include <klibc/preprocessor/variadics.h>
#include <klibc/preprocessor/seq.h>

#include <chaos/preprocessor/arithmetic/inc.h>
#include <chaos/preprocessor/arithmetic/dec.h>
#include <chaos/preprocessor/arithmetic/sub.h>
#include <chaos/preprocessor/facilities/empty.h>
#include <chaos/preprocessor/comparison/greater.h>
#include <chaos/preprocessor/detection/is_nullary.h>
#include <chaos/preprocessor/recursion/expr.h>
#include <chaos/preprocessor/repetition/repeat_from_to.h>
#include <chaos/preprocessor/control/if.h>
#include <chaos/preprocessor/control/variadic_if.h>
#include <chaos/preprocessor/control/while.h>
#include <chaos/preprocessor/logical/bool.h>
#include <chaos/preprocessor/logical/or.h>
#include <chaos/preprocessor/logical/not.h>
#include <chaos/preprocessor/tuple/core.h>
#include <chaos/preprocessor/tuple/drop.h>
#include <chaos/preprocessor/tuple/elem.h>
#include <chaos/preprocessor/lambda/ops.h>
#include <chaos/preprocessor/seq/core.h>
#include <chaos/preprocessor/seq/elem.h>
#include <chaos/preprocessor/seq/size.h>
#include <chaos/preprocessor/seq/fold_right.h>

#undef  __KLIBC_PP_BINARY_PREPARE_NUM
#define __KLIBC_PP_BINARY_PREPARE_NUM(num, toadd)   \
    CHAOS_PP_IF(CHAOS_PP_NOT(toadd))(               \
        num,                                        \
        KLIBC_PP_BINARY_BITS(                       \
            0, 0, CHAOS_PP_DEC(toadd)               \
        ) num                                       \
    )

#undef  __KLIBC_PP_BINARY_PREPARE
/**
 * @warning Binary numbers are assumed to be cleaned
 */
#define __KLIBC_PP_BINARY_PREPARE(first, second)            \
    CHAOS_PP_WHEN(                                          \
        CHAOS_PP_NOT(                                       \
            CHAOS_PP_OR(                                    \
                ISEMPTY(first)                              \
            )(                                              \
                ISEMPTY(second)                             \
            )                                               \
        )                                                   \
    )(                                                      \
        KLIBC_PP_EXPAND(                                    \
            CHAOS_PP_VARIADIC_IF(                           \
                CHAOS_PP_GREATER(                           \
                    __KLIBC_PP_BINARY_DIGITS(first),        \
                    __KLIBC_PP_BINARY_DIGITS(second)        \
                )                                           \
            )(                                              \
                first,                                      \
                __KLIBC_PP_BINARY_PREPARE_NUM(              \
                    second,                                 \
                    CHAOS_PP_SUB(                           \
                        __KLIBC_PP_BINARY_DIGITS(first),    \
                        __KLIBC_PP_BINARY_DIGITS(second)    \
                    )                                       \
                )                                           \
            )(                                              \
                __KLIBC_PP_BINARY_PREPARE_NUM(              \
                    first,                                  \
                    CHAOS_PP_SUB(                           \
                        __KLIBC_PP_BINARY_DIGITS(second),   \
                        __KLIBC_PP_BINARY_DIGITS(first)     \
                    )                                       \
                ),                                          \
                second                                      \
            )                                               \
        )                                                   \
    )                                                       \
    CHAOS_PP_WHEN(                                          \
        CHAOS_PP_AND(                                       \
            ISEMPTY(first)                                  \
        )(                                                  \
            ISEMPTY(second)                                 \
        )                                                   \
    )((0), (0))                                             \
    CHAOS_PP_WHEN(                                          \
        CHAOS_PP_AND(                                       \
            ISEMPTY(first)                                  \
        )(                                                  \
            CHAOS_PP_NOT(                                   \
                ISEMPTY(second)                             \
            )                                               \
        )                                                   \
    )(                                                      \
        KLIBC_PP_BINARY_BITS(                               \
            0, 0,                                           \
            CHAOS_PP_DEC(                                   \
                __KLIBC_PP_BINARY_DIGITS(second)            \
            )                                               \
        ),                                                  \
        second                                              \
    )                                                       \
    CHAOS_PP_WHEN(                                          \
        CHAOS_PP_AND(                                       \
            ISEMPTY(second)                                 \
        )(                                                  \
            CHAOS_PP_NOT(                                   \
                ISEMPTY(first)                              \
            )                                               \
        )                                                   \
    )(                                                      \
        first,                                              \
        KLIBC_PP_BINARY_BITS(                               \
            0, 0,                                           \
            CHAOS_PP_DEC(                                   \
                __KLIBC_PP_BINARY_DIGITS(first)             \
            )                                               \
        )                                                   \
    )

#undef  __KLIBC_PP_BINARY_DIGITS
/**
 * @warning Binary number is assumed to be cleaned
 */
#define __KLIBC_PP_BINARY_DIGITS(binary) \
    CHAOS_PP_IF(ISEMPTY(binary))(0, CHAOS_PP_SEQ_SIZE(binary))

#undef  KLIBC_PP_BINARY_DIGITS
#define KLIBC_PP_BINARY_DIGITS(binary) __KLIBC_PP_BINARY_DIGITS(KLIBC_PP_BINARY_CLEAN(binary))

#undef  KLIBC_PP_BINARY_OR
#define KLIBC_PP_BINARY_OR(first, second)       \
    __KLIBC_PP_BINARY_OR_IMPL(                  \
        KLIBC_PP_EXPAND(                        \
            __KLIBC_PP_BINARY_PREPARE(          \
                KLIBC_PP_BINARY_CLEAN(first),   \
                KLIBC_PP_BINARY_CLEAN(second)   \
            )                                   \
        )                                       \
    )

#undef  __KLIBC_PP_BINARY_OR_IMPL
#define __KLIBC_PP_BINARY_OR_IMPL(...)              \
    KLIBC_PP_FOR(                                   \
        0,                                          \
        __KLIBC_PP_BINARY_DIGITS(                   \
            CHAOS_PP_VARIADIC_ELEM(0, __VA_ARGS__)  \
        ),                                          \
        CHAOS_PP_LAMBDA(                            \
            (                                       \
                CHAOS_PP_OR_(                       \
                    CHAOS_PP_SEQ_ELEM_(             \
                        CHAOS_PP_ARG(3),            \
                        CHAOS_PP_VARIADIC_ELEM(     \
                            0, __VA_ARGS__          \
                        )                           \
                    )                               \
                )(                                  \
                    CHAOS_PP_SEQ_ELEM_(             \
                        CHAOS_PP_ARG(3),            \
                        CHAOS_PP_VARIADIC_ELEM(     \
                            1, __VA_ARGS__          \
                        )                           \
                    )                               \
                )                                   \
            )                                       \
        )                                           \
    )

#undef  KLIBC_PP_BINARY_CLEAN
#define KLIBC_PP_BINARY_CLEAN(binary)   \
    KLIBC_PP_SEQ_DROP_WHILE(            \
        CHAOS_PP_NOT_(                  \
            CHAOS_PP_SEQ_HEAD_(         \
                CHAOS_PP_ARG(1)         \
            )                           \
        ),                              \
        binary                          \
    )

#undef  KLIBC_PP_BINARY_SHIFTL
#define KLIBC_PP_BINARY_SHIFTL(binary, shift)   \
    CHAOS_PP_EXPR(                              \
        binary CHAOS_PP_REPEAT_FROM_TO(         \
            0,                                  \
            shift,                              \
            (0)                                 \
        )                                       \
    )

#undef  KLIBC_PP_BINARY_BITS
#define KLIBC_PP_BINARY_BITS(val, from, to) \
    CHAOS_PP_EXPR(                          \
        CHAOS_PP_REPEAT_FROM_TO(            \
            from, CHAOS_PP_INC(to),         \
            (CHAOS_PP_BOOL(val))            \
        )                                   \
    )                                       \
    CHAOS_PP_EXPR(                          \
        CHAOS_PP_REPEAT_FROM_TO(            \
            0, from,                        \
            (CHAOS_PP_NOT(val))             \
        )                                   \
    )

#undef  KLIBC_PP_BINARY
#define KLIBC_PP_BINARY(...)                    \
    __KLIBC_PP_BINARY_EXTRACT_RESULT(           \
        CHAOS_PP_EXPR(                          \
            CHAOS_PP_WHILE(                     \
                /* pred */                      \
                CHAOS_PP_NOT_(                  \
                    CHAOS_PP_LAMBDA(ISEMPTY)(   \
                        CHAOS_PP_REM_           \
                            CHAOS_PP_ARG(2)     \
                    )                           \
                ),                          \
                /* op */                    \
                __KLIBC_PP_BINARY_OP,       \
                /* result */                \
                (0),                        \
                /* args */                  \
                (__VA_ARGS__)               \
            )                               \
        )                                   \
    )
                
#undef  __KLIBC_PP_BINARY_OP
#define __KLIBC_PP_BINARY_OP(_, current_res, args)  \
    __KLIBC_PP_BINARY_OP_MK_NEW_RES(                \
        current_res,                                \
        CHAOS_PP_TUPLE_HEAD(args)                   \
    ),                                              \
    CHAOS_PP_IF(                                    \
        ISEMPTY(CHAOS_PP_TUPLE_TAIL(args))          \
    )(                                              \
        (),                                         \
        CHAOS_PP_TUPLE_TAIL(args)                   \
    )

#undef  __KLIBC_PP_BINARY_OP_MK_NEW_RES
#define __KLIBC_PP_BINARY_OP_MK_NEW_RES(                \
    current_res, current_arg                            \
)                                                       \
    KLIBC_PP_BINARY_OR(                                 \
        current_res,                                    \
        KLIBC_PP_BINARY_SHIFTL(                         \
            KLIBC_PP_BINARY_BITS(                       \
                CHAOS_PP_TUPLE_ELEM(                    \
                    0xunused, 0, current_arg            \
                ),                                      \
                0,                                      \
                CHAOS_PP_SUB(                           \
                    CHAOS_PP_TUPLE_ELEM(                \
                        0xunused, 1,                    \
                        CHAOS_PP_TUPLE_ELEM(            \
                            0xunused, 1, current_arg    \
                        )                               \
                    ),                                  \
                    CHAOS_PP_TUPLE_ELEM(                \
                        0xunused, 0,                    \
                        CHAOS_PP_TUPLE_ELEM(            \
                            0xunused, 1, current_arg    \
                        )                               \
                    )                                   \
                )                                       \
            ),                                          \
            CHAOS_PP_TUPLE_ELEM(                        \
                0xunused, 0,                            \
                CHAOS_PP_TUPLE_ELEM(                    \
                    0xunused, 1, current_arg            \
                )                                       \
            )                                           \
        )                                               \
    )

#undef  __KLIBC_PP_BINARY_EXTRACT_RESULT
#define __KLIBC_PP_BINARY_EXTRACT_RESULT(...) CHAOS_PP_VARIADIC_ELEM(0, __VA_ARGS__)

#undef  KLIBC_PP_BINARY_DEMOTE
#define KLIBC_PP_BINARY_DEMOTE(binary)  \
    CHAOS_PP_CAT(                       \
        0b,                             \
        CHAOS_PP_EXPR(                  \
            CHAOS_PP_SEQ_FOLD_RIGHT(    \
                CHAOS_PP_CAT_(          \
                    CHAOS_PP_ARG(1),    \
                    CHAOS_PP_ARG(2)     \
                ),                      \
                KLIBC_PP_BINARY_CLEAN(  \
                    binary              \
                ),                      \
                CHAOS_PP_EMPTY()        \
            )                           \
        )                               \
    )

#endif
