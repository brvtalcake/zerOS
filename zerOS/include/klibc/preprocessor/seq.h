#ifndef zerOS_KLIBC_PREPROCESSOR_SEQ_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_SEQ_H_INCLUDED

#include <chaos/preprocessor/seq/drop.h>
#include <chaos/preprocessor/recursion.h>
#include <chaos/preprocessor/control/while.h>

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

#endif
