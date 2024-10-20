
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

#if __SIZEOF_INT128__ == 16
#  if !defined(INT128_C) && CONFIG_LONG_LONG_IS_128_BIT
#    undef  __KLIBC_INT128_C_IMPL
#    define __KLIBC_INT128_C_IMPL(c)  c ## LL
#    define INT128_C(c, ...)  __KLIBC_INT128_C_IMPL(c)
#  endif
#
#  if !defined(UINT128_C) && CONFIG_LONG_LONG_IS_128_BIT
#    undef  __KLIBC_UINT128_C_IMPL
#    define __KLIBC_UINT128_C_IMPL(c)  c ## ULL
#    define UINT128_C(c, ...)  __KLIBC_UINT128_C_IMPL(c)
#  endif
#
# /* Inspired from https://github.com/arm-embedded/gcc-arm-none-eabi.debian/blob/master/src/gcc/testsuite/c-c%2B%2B-common/ubsan/float-cast.h#L14-L16 */
#
#  if !defined(INT128_MAX) && __CHAR_BIT__
#    define INT128_MAX (int128_t) (((uint128_t) 1 << ((__SIZEOF_INT128__ * __CHAR_BIT__) - 1)) - 1)
#  endif
#
#  if !defined(INT128_MIN) && defined(INT128_MAX)
#    define INT128_MIN (-INT128_MAX - 1)
#  endif
#
#  if !defined(UINT128_MAX) && defined(INT128_MAX)
#    define UINT128_MAX ((2 * (uint128_t) INT128_MAX) + 1)
#  endif
#else
#  error "__int128 is not 16 bytes long."
#endif

#endif
