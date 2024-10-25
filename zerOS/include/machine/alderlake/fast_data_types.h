#ifndef zerOS_MACHINE_ALDERLAKE_FAST_DATA_TYPES_H_INCLUDED
#define zerOS_MACHINE_ALDERLAKE_FAST_DATA_TYPES_H_INCLUDED

#include <machine/common/x86_64.h>

// TODO: Implement the following function with AVX2 instead
static inline void zerOS_fast_uint_set_vectorized(zerOS_fast_uint_t* array, size_t size, zerOS_fast_uint_t value)
{
    if (FAST_UINT_BITS(32))
    {
        const __m128i vector_value = _mm_set1_epi32(value);
        zerOS_fast_uint_t* array_aligned_up = (zerOS_fast_uint_t*)((uintptr_t)array & ~(16 - 1));

        // Fill the unaligned part
        while ((uintptr_t)array < (uintptr_t)array_aligned_up && size)
        {
            *array++ = value;
            size--;
        }

        // Fill the aligned part
        while (size >= 4)
        {
            _mm_store_si128((__m128i*)array, vector_value);
            array += 4;
            size -= 4;
        }

        // Fill the unaligned part
        while (size)
        {
            *array++ = value;
            size--;
        }

        return;
    }
    else if (FAST_UINT_BITS(64))
    {
        const __m128i vector_value = _mm_set1_epi64x(value);
        zerOS_fast_uint_t* array_aligned_up = (zerOS_fast_uint_t*)((uintptr_t)array & ~(16 - 1));

        // Fill the unaligned part
        while ((uintptr_t)array < (uintptr_t)array_aligned_up && size)
        {
            *array++ = value;
            size--;
        }

        // Fill the aligned part
        while (size >= 2)
        {
            _mm_store_si128((__m128i*)array, vector_value);
            array += 2;
            size -= 2;
        }

        // Fill the unaligned part
        while (size)
        {
            *array++ = value;
            size--;
        }

        return;
    }
    unreachable();
}

#endif
