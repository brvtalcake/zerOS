#ifndef zerOS_MACHINE_ALDERLAKE_FAST_DATA_TYPES_H_INCLUDED
#define zerOS_MACHINE_ALDERLAKE_FAST_DATA_TYPES_H_INCLUDED

#include <kernel/memory/align.h>

#include <machine/common/x86_64.h>

// TODO: Implement the following function with AVX2 instead
/**
 * @brief Set all elements of an array of `length` elements of type `zerOS_fast_uint_t` to a given
 * value, possibly using vectorized instructions.
 *
 * @param array The array to set.
 * @param length The length of the array.
 * @param value The value to set the array to.
 */
static void
zerOS_fast_uint_set_vectorized(zerOS_fast_uint_t* restrict array, size_t length, zerOS_fast_uint_t value)
{
    size_t remaining = length;
    if (FAST_UINT_BITS(32))
    {
        const __m128i vector_value     = _mm_set1_epi32(value);
        uintptr_t     array_aligned_up = zerOS_align_up((uintptr_t)array, 16);

        // Fill the unaligned part
        while ((uintptr_t)array < array_aligned_up && remaining)
        {
            *array++ = value;
            remaining--;
        }

        // Fill the aligned part
        while (remaining >= 4)
        {
            _mm_store_si128((__m128i*)array, vector_value);
            array     += 4;
            remaining -= 4;
        }

        // Fill the unaligned part
        while (remaining)
        {
            *array++ = value;
            remaining--;
        }

        return;
    }
    else if (FAST_UINT_BITS(64))
    {
        const __m128i vector_value     = _mm_set1_epi64x(value);
        uintptr_t     array_aligned_up = zerOS_align_up((uintptr_t)array, 16);

        // Fill the unaligned part
        while ((uintptr_t)array < array_aligned_up && remaining)
        {
            *array++ = value;
            remaining--;
        }

        // Fill the aligned part
        while (remaining >= 2)
        {
            _mm_store_si128((__m128i*)array, vector_value);
            array     += 2;
            remaining -= 2;
        }

        // Fill the unaligned part
        while (remaining)
        {
            *array++ = value;
            remaining--;
        }

        return;
    }
    unreachable();
}

#endif
