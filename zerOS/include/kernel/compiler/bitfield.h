#ifndef zerOS_KERNEL_COMPILER_BITFIELD_H_INCLUDED
#define zerOS_KERNEL_COMPILER_BITFIELD_H_INCLUDED

#include <stdint.h>

#include <chaos/preprocessor/control/when.h>
#include <chaos/preprocessor/logical/and.h>
#include <chaos/preprocessor/comparison/greater.h>
#include <chaos/preprocessor/comparison/less_equal.h>
#include <chaos/preprocessor/debug/failure.h>
#include <chaos/preprocessor/facilities/whitespace.h>

#undef  __BITFIELD_VALUE_CHOOSE_TYPE
#define __BITFIELD_VALUE_CHOOSE_TYPE(size)  \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_LESS_EQUAL(                \
            size,                           \
            8                               \
        )                                   \
    )(uint8_t)                              \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_AND                        \
        (                                   \
            CHAOS_PP_GREATER(               \
                size,                       \
                8                           \
            )                               \
        )(                                  \
            CHAOS_PP_LESS_EQUAL(            \
                size,                       \
                16                          \
            )                               \
        )                                   \
    )(uint16_t)                             \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_AND                        \
        (                                   \
            CHAOS_PP_GREATER(               \
                size,                       \
                16                          \
            )                               \
        )(                                  \
            CHAOS_PP_LESS_EQUAL(            \
                size,                       \
                32                          \
            )                               \
        )                                   \
    )(uint32_t)                             \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_AND                        \
        (                                   \
            CHAOS_PP_GREATER(               \
                size,                       \
                32                          \
            )                               \
        )(                                  \
            CHAOS_PP_LESS_EQUAL(            \
                size,                       \
                64                          \
            )                               \
        )                                   \
    )(uint64_t)                             \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_AND                        \
        (                                   \
            CHAOS_PP_GREATER(               \
                size,                       \
                64                          \
            )                               \
        )(                                  \
            CHAOS_PP_LESS_EQUAL(            \
                size,                       \
                128                         \
            )                               \
        )                                   \
    )(uint128_t)                            \
    CHAOS_PP_WHEN(                          \
        CHAOS_PP_GREATER(size, 128)         \
    )(CHAOS_PP_FAILURE())

#undef  BITFIELD_VALUE
#ifndef __INTELLISENSE__
    #define BITFIELD_VALUE(name, size) CHAOS_PP_CLEAN(__BITFIELD_VALUE_CHOOSE_TYPE(size)) name : size
#else
    #define BITFIELD_VALUE(name, size) uint128_t name : size
#endif

#endif