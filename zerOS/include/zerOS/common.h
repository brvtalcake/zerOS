#ifndef zerOS_COMMON_H_INCLUDED_
#define zerOS_COMMON_H_INCLUDED_ 1

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __INTELLISENSE__
	#include <stdalign.h>
	#ifndef constexpr
		#define constexpr const
	#endif
	#ifndef nullptr
		#define nullptr NULL
	#endif
	#ifndef static_assert
		#define static_assert(...) _Static_assert(__VA_ARGS__)
	#endif
	#ifndef alignas
		#define alignas _Alignas
	#endif
	#ifndef alignof
		#define alignof _Alignof
	#endif
#endif

typedef uint8_t __attribute__((__may_alias__)) zerOS_byte_t;
static_assert(sizeof(zerOS_byte_t) == 1);
static_assert(alignof(zerOS_byte_t) == 1);

#undef container_of
#define container_of(type, pointer, member) \
	((type*)((zerOS_byte_t*)pointer - offsetof(type, member)))

#ifndef auto
	#define auto __auto_type
#endif

// TODO: custom, optimized, memcpy/memmove/etc...
// see:
// - https://www.codeproject.com/Articles/1110153/Apex-memmove-the-fastest-memcpy-memmove-on-x-x-EVE
// - https://blog.magnum.graphics/backstage/cpu-feature-detection-dispatch/
extern void* memmove(void* dst, const void* src, size_t size);
extern void* memcpy(void* restrict dst, const void* restrict src, size_t size);

#undef inline_memcpy
#define inline_memcpy(destination, source, size)                  \
	({                                                            \
		__builtin_memcpy_inline((destination), (source), (size)); \
		(destination);                                            \
	})

#undef inline_memset
#define inline_memset(destination, value, size)                  \
	({                                                           \
		__builtin_memset_inline((destination), (value), (size)); \
		(destination);                                           \
	})

#endif
