#ifndef zerOS_MACHINE_COMMON_X86_64_H_INCLUDED
#define zerOS_MACHINE_COMMON_X86_64_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

typedef uint_fast32_t zerOS_fast_uint_t;

static constexpr size_t zerOS_fast_uint_size = sizeof(zerOS_fast_uint_t);
static constexpr size_t zerOS_fast_uint_bits = zerOS_fast_uint_size * 8;

static constexpr zerOS_fast_uint_t zerOS_fast_uint_max = UINT_FAST32_MAX;

#endif
