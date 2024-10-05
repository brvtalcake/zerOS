#ifndef zerOS_KLIBC_PREPROCESSOR_FOR_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_FOR_H_INCLUDED

#include <chaos/preprocessor/arbitrary/inc.h>
#include <chaos/preprocessor/arbitrary/not_equal.h>
#include <chaos/preprocessor/arbitrary/promote.h>
#include <chaos/preprocessor/arbitrary/demote.h>
#include <chaos/preprocessor/arithmetic/inc.h>
#include <chaos/preprocessor/comparison/not_equal.h>
#include <chaos/preprocessor/lambda/ops.h>
#include <chaos/preprocessor/lambda/invoke.h>
#include <chaos/preprocessor/recursion/expr.h>
#include <chaos/preprocessor/repetition/repeat_from_to.h>
#include <chaos/preprocessor/repetition/for.h>
#include <chaos/preprocessor/control/variadic_if.h>

#include <klibc/preprocessor/lambda.h>

#undef  __KLIBC_PP_FOR_MAYBE_INVOKE
#define __KLIBC_PP_FOR_MAYBE_INVOKE(pplambda, from, to, ...)    \
    CHAOS_PP_VARIADIC_IF(KLIBC_PP_IS_LAMBDA(pplambda))(         \
        CHAOS_PP_INVOKE_(                                       \
            pplambda, from, to, CHAOS_PP_ARG(3), __VA_ARGS__    \
        )                                                       \
    )(                                                          \
        CHAOS_PP_LAMBDA(pplambda)(                              \
            from, to, CHAOS_PP_ARG(3), __VA_ARGS__              \
        )                                                       \
    )
        

#undef  KLIBC_PP_FOR
#define KLIBC_PP_FOR(from, to, pplambda, ...)   \
    CHAOS_PP_EXPR(                              \
        CHAOS_PP_FOR(                           \
            /* pred */                          \
            CHAOS_PP_NOT_EQUAL_(                \
                CHAOS_PP_ARG(3),                \
                CHAOS_PP_ARG(2)                 \
            ),                                  \
            /* op */                            \
            CHAOS_PP_LAMBDA(                    \
                from,                           \
                to,                             \
                CHAOS_PP_INC_(                  \
                    CHAOS_PP_ARG(3)             \
                )                               \
            ),                                  \
            /* macro */                         \
            __KLIBC_PP_FOR_MAYBE_INVOKE(        \
                pplambda, from, to, __VA_ARGS__ \
            ),                                  \
            /* state */                         \
            from, to, from                      \
        )                                       \
    )

#undef  KLIBC_PP_FOR_EXTENDED
#define KLIBC_PP_FOR_EXTENDED(from, to, pplambda, ...)  \
    CHAOS_PP_EXPR(                                      \
        CHAOS_PP_FOR(                                   \
            /* pred */                                  \
            CHAOS_PP_ARBITRARY_NOT_EQUAL_(              \
                CHAOS_PP_ARG(3),                        \
                CHAOS_PP_ARG(2)                         \
            ),                                          \
            /* op */                                    \
            CHAOS_PP_LAMBDA(                            \
                from,                                   \
                to,                                     \
                CHAOS_PP_ARBITRARY_INC_(                \
                    CHAOS_PP_ARG(3)                     \
                )                                       \
            ),                                          \
            /* macro */                                 \
            __KLIBC_PP_FOR_MAYBE_INVOKE(                \
                pplambda, from, to, __VA_ARGS__         \
            ),                                          \
            /* state */                                 \
            from, to, from                              \
        )                                               \
    )

#endif
