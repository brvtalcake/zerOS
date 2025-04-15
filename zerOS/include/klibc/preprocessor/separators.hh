#ifndef zerOS_KLIBC_PREPROCESSOR_SEPARATORS_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_SEPARATORS_H_INCLUDED

#include <chaos/preprocessor/control/when.h>

#undef  KLIBC_PP_SEP_IF
/**
 * @def KLIBC_PP_SEP_IF(cond)(sep)
 * @brief Conditionally inserts a separator.
 * @param cond The condition.
 * @param sep  The separator.
 * @return     The separator if the condition is true, otherwise nothing.
 */
#define KLIBC_PP_SEP_IF(cond) CHAOS_PP_WHEN(cond)

#endif
