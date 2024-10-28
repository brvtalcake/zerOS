#ifndef zerOS_MACHINE_COMMON_X86_64_H_INCLUDED
#define zerOS_MACHINE_COMMON_X86_64_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <x86intrin.h>

#include <klibc/misc.h>

typedef uint_fast32_t zerOS_fast_uint_t;

#undef  FAST_UINT_BITS
#if 0
// Fixme
#define FAST_UINT_BITS(bits) KLIBC_TYPES_EQUAL(zerOS_fast_uint_t, uint##bits##_t)
#else
#define FAST_UINT_BITS(bits) (zerOS_fast_uint_bits == (bits))
#endif

static constexpr size_t zerOS_fast_uint_size = sizeof(zerOS_fast_uint_t);
static constexpr size_t zerOS_fast_uint_bits = zerOS_fast_uint_size * 8;

static constexpr zerOS_fast_uint_t zerOS_fast_uint_max = UINT_FAST32_MAX;

static_assert(
    ( FAST_UINT_BITS(32) && !FAST_UINT_BITS(64)) ||
    (!FAST_UINT_BITS(32) &&  FAST_UINT_BITS(64)) ,
    "zerOS: fast uint should be either 32 or 64 bits"
);

#endif
