#ifndef zerOS_KLIBC_PREPROCESSOR_DEFAULT_ARG_H_INCLUDED
#define zerOS_KLIBC_PREPROCESSOR_DEFAULT_ARG_H_INCLUDED

#include <chaos/preprocessor/control/variadic_if.h>
#include <chaos/preprocessor/tuple/rem.h>
#include <klibc/preprocessor/empty.h>

#undef  KLIBC_PP_DEFAULT_ARG
#define KLIBC_PP_DEFAULT_ARG(default, ...) CHAOS_PP_VARIADIC_IF(ISEMPTY(__VA_ARGS__))(default)(__VA_ARGS__)

#undef  KLIBC_PP_DEFAULT_ARGS
#define KLIBC_PP_DEFAULT_ARGS(defaults, ...) CHAOS_PP_VARIADIC_IF(ISEMPTY(__VA_ARGS__))(CHAOS_PP_REM defaults)(__VA_ARGS__)

#endif
