#ifndef zerOS_KLIBC_PREPROCESSOR_SEQ_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_SEQ_H_INCLUDED

#include <pp_empty/pp_is_empty.h>

#include <klibc/preprocessor/variadics.h>
#include <klibc/preprocessor/tuple.h>
#include <klibc/preprocessor/separators.h>
#include <klibc/preprocessor/lambda.h>

#include <chaos/preprocessor/punctuation.h>
#include <chaos/preprocessor/tuple/rem.h>
#include <chaos/preprocessor/tuple/size.h>
#include <chaos/preprocessor/lambda/ops.h>
#include <chaos/preprocessor/logical/not.h>
#include <chaos/preprocessor/logical/bool.h>
#include <chaos/preprocessor/seq/elem.h>
#include <chaos/preprocessor/seq/size.h>
#include <chaos/preprocessor/seq/drop.h>
#include <chaos/preprocessor/seq/variadic.h>
#include <chaos/preprocessor/recursion.h>
#include <chaos/preprocessor/repetition/for.h>
#include <chaos/preprocessor/control/while.h>
#include <chaos/preprocessor/control/variadic_if.h>
#include <chaos/preprocessor/comparison/equal.h>
#include <chaos/preprocessor/comparison/less.h>
#include <chaos/preprocessor/debug/assert.h>
#include <chaos/preprocessor/arithmetic/inc.h>

#undef  KLIBC_PP_SEQ_DROP_WHILE
/**
 * @def KLIBC_PP_SEQ_DROP_WHILE(pred, seq)
 * @brief Drops elements from the beginning of a sequence while the predicate is true.
 * @param pred A predicate.
 * @param seq  A sequence.
 * @return     The sequence with elements dropped.
 */
#define KLIBC_PP_SEQ_DROP_WHILE(pred, seq)  \
    CHAOS_PP_EXPR(                          \
        CHAOS_PP_WHILE(                     \
            pred,                           \
            CHAOS_PP_SEQ_DROP_(             \
                1, CHAOS_PP_ARG(1)          \
            ),                              \
            seq                             \
        )                                   \
    )

#undef  __KLIBC_PP_SEQ_ZIP_IMPL_2SEQ
#define __KLIBC_PP_SEQ_ZIP_IMPL_2SEQ(seq1, seq2)        \
    CHAOS_PP_ASSERT_MSG(                                \
        CHAOS_PP_EQUAL(                                 \
            CHAOS_PP_SEQ_SIZE_ALT(seq1),                \
            CHAOS_PP_SEQ_SIZE_ALT(seq2)                 \
        ),                                              \
        "KLIBC_PP_SEQ_ZIP: "                            \
        "Sequences must have the same size."            \
    )                                                   \
    CHAOS_PP_EXPR(                                      \
        CHAOS_PP_FOR(                                   \
            CHAOS_PP_LESS_(                             \
                CHAOS_PP_ARG(3),                        \
                CHAOS_PP_ARG(4)                         \
            ),                                          \
            CHAOS_PP_LAMBDA(                            \
                CHAOS_PP_ARG(1),                        \
                CHAOS_PP_ARG(2),                        \
                CHAOS_PP_INC_(CHAOS_PP_ARG(3)),         \
                CHAOS_PP_ARG(4)                         \
            ),                                          \
            CHAOS_PP_LAMBDA(                            \
                (                                       \
                    CHAOS_PP_SEQ_ELEM_(                 \
                        CHAOS_PP_ARG(3),                \
                        CHAOS_PP_ARG(1)                 \
                    ),                                  \
                    CHAOS_PP_SEQ_ELEM_(                 \
                        CHAOS_PP_ARG(3),                \
                        CHAOS_PP_ARG(2)                 \
                    )                                   \
                )                                       \
            ),                                          \
            seq1, seq2, 0, CHAOS_PP_SEQ_SIZE_ALT(seq1)  \
        )                                               \
    )

#undef  __KLIBC_PP_SEQ_ZIP_IMPL_MACPRED
#define __KLIBC_PP_SEQ_ZIP_IMPL_MACPRED(_, seq1, others) CHAOS_PP_BOOL(CHAOS_PP_TUPLE_SIZE(others))

#undef  __KLIBC_PP_SEQ_ZIP_IMPL_MACOP
#define __KLIBC_PP_SEQ_ZIP_IMPL_MACOP(_, seq1, others)  \
    __KLIBC_PP_SEQ_ZIP_IMPL_2SEQ(                       \
        seq1,                                           \
        CHAOS_PP_TUPLE_HEAD(others)                     \
    ),                                                  \
    CHAOS_PP_TUPLE_TAIL(others)

#undef  __KLIBC_PP_SEQ_ZIP_IMPL
#define __KLIBC_PP_SEQ_ZIP_IMPL(seq1, ...)              \
    CHAOS_PP_VARIADIC_ELEM(                             \
        0,                                              \
        CHAOS_PP_EXPR(                                  \
            CHAOS_PP_WHILE(                             \
                /* */                                   \
                __KLIBC_PP_SEQ_ZIP_IMPL_MACPRED,        \
                /* */                                   \
                __KLIBC_PP_SEQ_ZIP_IMPL_MACOP,          \
                /* */                                   \
                seq1, (__VA_ARGS__)                     \
            )                                           \
        )                                               \
    )

#undef  KLIBC_PP_SEQ_ZIP
/**
 * @def KLIBC_PP_SEQ_ZIP(seq1, ... seqN)
 * @brief Zips Multiple sequences into a single (variadic) preprocessor sequence.
 * @param seq1 The first sequence.
 * @param ...  The rest of the sequences.
 * @return     The zipped sequence, i.e. (seq1[0], seq2[0], ..., seqN[0])(seq1[1], seq2[1], ..., seqN[1])...(seq1[N], seq2[N], ..., seqN[N]).
 */
#define KLIBC_PP_SEQ_ZIP(seq1, ...) \
    CHAOS_PP_VARIADIC_IF(           \
        KLIBC_PP_VA_COUNT(          \
            __VA_ARGS__             \
        )                           \
    )(                              \
        __KLIBC_PP_SEQ_ZIP_IMPL(    \
            seq1, __VA_ARGS__       \
        )                           \
    )(seq1)

#undef  __KLIBC_PP_SEQ_MAP_IMPL_ID
#define __KLIBC_PP_SEQ_MAP_IMPL_ID() __KLIBC_PP_SEQ_MAP_IMPL

#undef  __KLIBC_PP_SEQ_MAP_IMPL_END
#define __KLIBC_PP_SEQ_MAP_IMPL_END CHAOS_PP_EAT

#undef  __KLIBC_PP_SEQ_MAP_IMPL
#define __KLIBC_PP_SEQ_MAP_IMPL(sep, macro, variadic_seq, invoc_num)    \
    CHAOS_PP_EXPR(                                                      \
        CHAOS_PP_VARIADIC_IF(                                           \
            CHAOS_PP_LESS(                                              \
                invoc_num,                                              \
                CHAOS_PP_SEQ_SIZE_ALT(variadic_seq)                     \
            )                                                           \
        )(                                                              \
            KLIBC_PP_SEP_IF(invoc_num)(KLIBC_PP_LAMBDA(sep)())          \
            CHAOS_PP_CALL(macro)()(                                     \
                CHAOS_PP_STATE(), macro,                                \
                CHAOS_PP_SEQ_ELEM(                                      \
                    invoc_num,                                          \
                    variadic_seq                                        \
                )                                                       \
            ) CHAOS_PP_OBSTRUCT(                                        \
                __KLIBC_PP_SEQ_MAP_IMPL_ID                              \
            )()                                                         \
        )(                                                              \
            __KLIBC_PP_SEQ_MAP_IMPL_END                                 \
        )(                                                              \
            sep, macro, variadic_seq, CHAOS_PP_INC(invoc_num)           \
        )                                                               \
    )

#undef  KLIBC_PP_SEQ_MAP
/**
 * @def KLIBC_PP_SEQ_MAP(sep, macro, seq)
 * @brief Maps a macro over a sequence.
 * @param sep   The separator.
 * @param macro The macro to map.
 * @param seq   The sequence.
 * @param ...   Other sequences.
 * @return      The mapped sequence.
 */
#define KLIBC_PP_SEQ_MAP(sep, macro, seq, ...)      \
    __KLIBC_PP_SEQ_MAP_IMPL(                        \
        macro, KLIBC_PP_SEQ_ZIP(seq, __VA_ARGS__)   \
    )

#endif
