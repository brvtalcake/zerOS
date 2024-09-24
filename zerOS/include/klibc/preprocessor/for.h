#ifndef zerOS_KLIBC_PREPROCESSOR_FOR_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_FOR_H_INCLUDED

#include <chaos/preprocessor/arbitrary.h>
#include <chaos/preprocessor/lambda/ops.h>
#include <chaos/preprocessor/lambda/invoke.h>
#include <chaos/preprocessor/recursion/expr.h>
#include <chaos/preprocessor/repetition/repeat_from_to.h>
#include <chaos/preprocessor/repetition/for.h>

#undef  KLIBC_PP_FOR
#define KLIBC_PP_FOR(from, to, pplambda, ...)   \
    CHAOS_PP_EXPR(                              \
        CHAOS_PP_REPEAT_FROM_TO(                \
            from, to, pplambda, __VA_ARGS__     \
        )                                       \
    )

#undef  KLIBC_PP_FOR_EXTENDED
#define KLIBC_PP_FOR_EXTENDED(from, to, pplambda, ...)  \
    CHAOS_PP_EXPR(                                      \
        CHAOS_PP_FOR(                                   \
            CHAOS_PP_ARBITRARY_EQUAL_(                  \
                CHAOS_PP_ARG(0),                        \
                CHAOS_PP_ARG(1)                         \
            ),                                          \
            CHAOS_PP_LAMBDA(                            \
                CHAOS_PP_ARBITRARY_INC_(                \
                    CHAOS_PP_ARG(0)                     \
                ),                                      \
                CHAOS_PP_ARG(1),                        \
                CHAOS_PP_ARG(2),                        \
                CHAOS_PP_ARG(3)                         \
            ),                                          \
            CHAOS_PP_LAMBDA(                            \
                CHAOS_PP_INVOKE_(                       \
                    CHAOS_PP_ARG(2),                    \
                    CHAOS_PP_ARG(0),                    \
                    CHAOS_PP_REM_ CHAOS_PP_ARG(3)       \
                )                                       \
            ),                                          \
            from, to, pplambda, (__VA_ARGS__)           \
        )                                               \
    )

#endif

KLIBC_PP_FOR_EXTENDED(
    CHAOS_PP_ARBITRARY_PROMOTE(0),
    CHAOS_PP_ARBITRARY_PROMOTE(10),
    CHAOS_PP_LAMBDA(
        CHAOS_PP_ARBITRARY_DEMOTE_(CHAOS_PP_ARG(0)) CHAOS_PP_LAMBDA(+) CHAOS_PP_ARBITRARY_DEMOTE_(CHAOS_PP_ARG(1)) CHAOS_PP_COMMA_IF_(CHAOS_PP_ARBITRARY_NOT_EQUAL_(CHAOS_PP_ARG(0), CHAOS_PP_ARG(1)))
    ),
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10
)
