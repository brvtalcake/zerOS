
#ifdef __has_include_next
#  if __has_include_next(<stdint.h>)
#    include_next <stdint.h>
#  else
#     error "Cannot include the next (standard) <stdint.h> header."
#  endif
#else
#  error    "Cannot include the next (standard) <stdint.h> header."
#endif

#ifndef zerOS_STDINT_H_INCLUDED
#define zerOS_STDINT_H_INCLUDED

#include <config.h>

typedef __int128_t  int128_t;
typedef __uint128_t uint128_t;

#include <klibc/misc.h>

#if !defined(INT128_C) && CONFIG_LONG_LONG_IS_128_BIT
#  undef  __KLIBC_INT128_C_IMPL
#  define __KLIBC_INT128_C_IMPL(c)  c ## LL
#  define INT128_C(c)  __KLIBC_INT128_C_IMPL(c)
#endif

#if !defined(UINT128_C) && CONFIG_LONG_LONG_IS_128_BIT
#  undef  __KLIBC_UINT128_C_IMPL
#  define __KLIBC_UINT128_C_IMPL(c)  c ## ULL
#  define UINT128_C(c)  __KLIBC_UINT128_C_IMPL(c)
#endif

#undef  LITMK
#if defined(INT128_C) && defined(UINT128_C)
#  define LITMK(type, x)        \
    _Generic((type)(0),         \
        int8_t:    INT8_C(x)  , \
        int16_t:   INT16_C(x) , \
        int32_t:   INT32_C(x) , \
        int64_t:   INT64_C(x) , \
        int128_t:  INT128_C(x), \
        uint8_t:   UINT8_C(x) , \
        uint16_t:  UINT16_C(x), \
        uint32_t:  UINT32_C(x), \
        uint64_t:  UINT64_C(x), \
        uint128_t: UINT128_C(x) \
    )
#else
#  define LITMK(type, x)        \
    _Generic((type)(0),         \
        int8_t:    INT8_C(x)  , \
        int16_t:   INT16_C(x) , \
        int32_t:   INT32_C(x) , \
        int64_t:   INT64_C(x) , \
        int128_t:  (type)0    , \
        uint8_t:   UINT8_C(x) , \
        uint16_t:  UINT16_C(x), \
        uint32_t:  UINT32_C(x), \
        uint64_t:  UINT64_C(x), \
        uint128_t: (type)0      \
    )
#endif

#endif
