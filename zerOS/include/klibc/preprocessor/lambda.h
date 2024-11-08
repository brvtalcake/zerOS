#ifndef zerOS_KLIBC_PREPROCESSOR_LAMBDA_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_LAMBDA_H_INCLUDED

#include <chaos/preprocessor/lambda/ops.h>
#include <chaos/preprocessor/detection/compare.h>
#include <chaos/preprocessor/control/if.h>

#include <klibc/preprocessor/empty.h>

#undef  KLIBC_PP_LAMBDA
#define KLIBC_PP_LAMBDA(...) CHAOS_PP_LAMBDA(__VA_ARGS__)

#undef  KLIBC_PP_ARG0
#undef  KLIBC_PP_ARG1
#undef  KLIBC_PP_ARG2
#undef  KLIBC_PP_ARG3
#undef  KLIBC_PP_ARG4
#undef  KLIBC_PP_ARG5
#undef  KLIBC_PP_ARG6
#undef  KLIBC_PP_ARG7
#undef  KLIBC_PP_ARG8
#undef  KLIBC_PP_ARG9
#undef  KLIBC_PP_ARG10
#undef  KLIBC_PP_ARG11
#undef  KLIBC_PP_ARG12
#undef  KLIBC_PP_ARG13
#undef  KLIBC_PP_ARG14
#undef  KLIBC_PP_ARG15
#undef  KLIBC_PP_ARG16
#undef  KLIBC_PP_ARG17
#undef  KLIBC_PP_ARG18
#undef  KLIBC_PP_ARG19
#undef  KLIBC_PP_ARG20
#undef  KLIBC_PP_ARG21
#undef  KLIBC_PP_ARG22
#undef  KLIBC_PP_ARG23
#undef  KLIBC_PP_ARG24
#define KLIBC_PP_ARG0  CHAOS_PP_ARG(0)
#define KLIBC_PP_ARG1  CHAOS_PP_ARG(1)
#define KLIBC_PP_ARG2  CHAOS_PP_ARG(2)
#define KLIBC_PP_ARG3  CHAOS_PP_ARG(3)
#define KLIBC_PP_ARG4  CHAOS_PP_ARG(4)
#define KLIBC_PP_ARG5  CHAOS_PP_ARG(5)
#define KLIBC_PP_ARG6  CHAOS_PP_ARG(6)
#define KLIBC_PP_ARG7  CHAOS_PP_ARG(7)
#define KLIBC_PP_ARG8  CHAOS_PP_ARG(8)
#define KLIBC_PP_ARG9  CHAOS_PP_ARG(9)
#define KLIBC_PP_ARG10 CHAOS_PP_ARG(10)
#define KLIBC_PP_ARG11 CHAOS_PP_ARG(11)
#define KLIBC_PP_ARG12 CHAOS_PP_ARG(12)
#define KLIBC_PP_ARG13 CHAOS_PP_ARG(13)
#define KLIBC_PP_ARG14 CHAOS_PP_ARG(14)
#define KLIBC_PP_ARG15 CHAOS_PP_ARG(15)
#define KLIBC_PP_ARG16 CHAOS_PP_ARG(16)
#define KLIBC_PP_ARG17 CHAOS_PP_ARG(17)
#define KLIBC_PP_ARG18 CHAOS_PP_ARG(18)
#define KLIBC_PP_ARG19 CHAOS_PP_ARG(19)
#define KLIBC_PP_ARG20 CHAOS_PP_ARG(20)
#define KLIBC_PP_ARG21 CHAOS_PP_ARG(21)
#define KLIBC_PP_ARG22 CHAOS_PP_ARG(22)
#define KLIBC_PP_ARG23 CHAOS_PP_ARG(23)
#define KLIBC_PP_ARG24 CHAOS_PP_ARG(24)

#undef  CHAOS_PP_COMPARE_0xLAMBDA
#define CHAOS_PP_COMPARE_0xLAMBDA(x) x

#undef  KLIBC_PP_IS_LAMBDA
#define KLIBC_PP_IS_LAMBDA(...)                 \
    CHAOS_PP_IF(                                \
        ISEMPTY(                                \
            CHAOS_PP_FLAG_NAME(__VA_ARGS__)     \
        )                                       \
    )(                                          \
        0,                                      \
        CHAOS_PP_COMPARE(                       \
            0xLAMBDA,                           \
            CHAOS_PP_FLAG_NAME(__VA_ARGS__)     \
        )                                       \
    )

#undef  KLIBC_PP_MAKE_LAMBDA
#define KLIBC_PP_MAKE_LAMBDA(...)   \
    CHAOS_PP_VARIADIC_IF(           \
        KLIBC_PP_IS_LAMBDA(         \
            __VA_ARGS__             \
        )                           \
    )(                              \
        __VA_ARGS__                 \
    )(CHAOS_PP_LAMBDA(__VA_ARGS__))


#undef  KLIBC_PP_LAMBDA_COMPOSE
#undef  __KLIBC_PP_LAMBDA_COMPOSE2
/* TODO */

#endif
