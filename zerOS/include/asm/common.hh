#ifndef zerOS_ASM_COMMON_H_INCLUDED
#define zerOS_ASM_COMMON_H_INCLUDED

#ifdef __ASSEMBLER__

#include <asm/syntax.h>

#include <klibc/preprocessor/empty.h>

#include <chaos/preprocessor/control/variadic_if.h>
#include <chaos/preprocessor/control/when.h>
#include <chaos/preprocessor/control/unless.h>
#include <chaos/preprocessor/debug/failure.h>
#include <chaos/preprocessor/logical/bool.h>
#include <chaos/preprocessor/logical/or.h>
#include <chaos/preprocessor/comparison/equal.h>

#undef   ACCESS_PTR

#undef  __ACCESS_PTR_ATT
#undef  __ACCESS_PTR_INTEL

#undef  __ACCESS_PTR_ATT_MAYBE_EXPAND_IDXSCL
#undef  __ACCESS_PTR_INTEL_MAYBE_EXPAND_IDXSCL
#undef  __ACCESS_PTR_INTEL_MAYBE_EXPAND_DISP

#define __ACCESS_PTR_ATT_MAYBE_EXPAND_IDXSCL(index, scale)  \
    CHAOS_PP_UNLESS(                                        \
        CHAOS_PP_OR(ISEMPTY(index))(ISEMPTY(scale))         \
    )( , index, scale )
#define __ACCESS_PTR_INTEL_MAYBE_EXPAND_IDXSCL(index, scale)    \
    CHAOS_PP_UNLESS(                                            \
        CHAOS_PP_OR(ISEMPTY(index))(ISEMPTY(scale))             \
    )( + index * scale )
#define __ACCESS_PTR_INTEL_MAYBE_EXPAND_DISP(displacement)  \
    CHAOS_PP_UNLESS(                                        \
        ISEMPTY(displacement)                               \
    )( + displacement)

#define __ACCESS_PTR_ATT(section, base, index, scale, displacement) \
    CHAOS_PP_VARIADIC_IF(                                           \
        ISEMPTY(section)                                            \
    )(                                                              \
        displacement(                                               \
            base __ACCESS_PTR_ATT_MAYBE_EXPAND_IDXSCL(              \
                index, scale                                        \
            )                                                       \
        )                                                           \
    )(                                                              \
        section:displacement(                                       \
            base __ACCESS_PTR_ATT_MAYBE_EXPAND_IDXSCL(              \
                index, scale                                        \
            )                                                       \
        )                                                           \
    )
#define __ACCESS_PTR_INTEL(section, base, index, scale, displacement) \
    CHAOS_PP_VARIADIC_IF(                                             \
        ISEMPTY(section)                                              \
    )(                                                                \
        [                                                             \
            base                                                      \
            __ACCESS_PTR_INTEL_MAYBE_EXPAND_IDXSCL(index, scale)      \
            __ACCESS_PTR_INTEL_MAYBE_EXPAND_DISP(displacement)        \
        ]                                                             \
    )(                                                                \
        [                                                             \
            section:base                                              \
            __ACCESS_PTR_INTEL_MAYBE_EXPAND_IDXSCL(index, scale)      \
            __ACCESS_PTR_INTEL_MAYBE_EXPAND_DISP(displacement)        \
        ]                                                             \
    )

/**
 * @def ACCESS_PTR
 * @brief Access a pointer.
 * @param section The section in which the pointer is located.
 * @param base The base pointer.
 * @param index The index.
 * @param scale The scale.
 * @param displacement The displacement.
 * @note The syntax is determined by the value of `ASM_SYNTAX` (either `ASM_SYNTAX_ATT` or `ASM_SYNTAX_INTEL`).
 * @note The syntax is undefined if `base` is empty.
 */
#define ACCESS_PTR(section, base, index, scale, displacement)   \
    CHAOS_PP_VARIADIC_IF(                                       \
        CHAOS_PP_OR(ISEMPTY(base))(ASM_SYNTAX_UNDEFINED)        \
    )(                                                          \
        CHAOS_PP_FAILURE()                                      \
    )(                                                          \
        CHAOS_PP_WHEN(                                          \
            CHAOS_PP_EQUAL(                                     \
                ASM_SYNTAX,                                     \
                ASM_SYNTAX_INTEL                                \
            )                                                   \
        )(__ACCESS_PTR_INTEL)                                   \
        CHAOS_PP_WHEN(                                          \
            CHAOS_PP_EQUAL(                                     \
                ASM_SYNTAX,                                     \
                ASM_SYNTAX_ATT                                  \
            )                                                   \
        )(__ACCESS_PTR_ATT)                                     \
    )(section, base, index, scale, displacement)

#undef  RIP_RELATIVE
/**
 * @def RIP_RELATIVE
 * @brief Access a pointer relative to the instruction pointer.
 * @param symbol The symbol to get the relative address of.
 * @note The syntax is determined by the value of `ASM_SYNTAX` (either `ASM_SYNTAX_ATT` or `ASM_SYNTAX_INTEL`).
 */
#define RIP_RELATIVE(symbol)                        \
    CHAOS_PP_VARIADIC_IF(                           \
        CHAOS_PP_OR(                                \
            ISEMPTY(symbol)                         \
        )(                                          \
            ASM_SYNTAX_UNDEFINED                    \
        )                                           \
    )(                                              \
        CHAOS_PP_FAILURE()                          \
    )(                                              \
        CHAOS_PP_WHEN(                              \
            CHAOS_PP_EQUAL(                         \
                ASM_SYNTAX,                         \
                ASM_SYNTAX_INTEL                    \
            )                                       \
        )([ rel symbol ])                           \
        CHAOS_PP_WHEN(                              \
            CHAOS_PP_EQUAL(                         \
                ASM_SYNTAX,                         \
                ASM_SYNTAX_ATT                      \
            )                                       \
        )(symbol(%rip))                             \
    )

#endif

#endif