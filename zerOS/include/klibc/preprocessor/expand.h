#ifndef zerOS_KLIBC_PREPROCESSOR_EXPAND_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_EXPAND_H_INCLUDED

#include <chaos/preprocessor/facilities/expand.h>

#undef  KLIBC_PP_EXPAND
/**
 * @def KLIBC_PP_EXPAND(...)
 * @brief Expands the arguments.
 * @param ... The arguments.
 * @return The expanded arguments.
 */
#define KLIBC_PP_EXPAND(...) __KLIBC_PP_EXPAND_I(__KLIBC_PP_EXPAND_I(__KLIBC_PP_EXPAND_I(__KLIBC_PP_EXPAND_I(__KLIBC_PP_EXPAND_I(__KLIBC_PP_EXPAND_I(__VA_ARGS__))))))

#undef  __KLIBC_PP_EXPAND_I
#undef  __KLIBC_PP_EXPAND_II
#define __KLIBC_PP_EXPAND_I(...) __KLIBC_PP_EXPAND_II(__KLIBC_PP_EXPAND_II(__KLIBC_PP_EXPAND_II(__KLIBC_PP_EXPAND_II(__KLIBC_PP_EXPAND_II(__KLIBC_PP_EXPAND_II(__VA_ARGS__))))))
#define __KLIBC_PP_EXPAND_II(...) CHAOS_PP_EXPAND(CHAOS_PP_EXPAND(CHAOS_PP_EXPAND(CHAOS_PP_EXPAND(CHAOS_PP_EXPAND(CHAOS_PP_EXPAND(__VA_ARGS__))))))

#endif