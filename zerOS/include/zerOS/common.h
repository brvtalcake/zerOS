#ifndef zerOS_COMMON_H_INCLUDED_
#define zerOS_COMMON_H_INCLUDED_ 1

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __INTELLISENSE__
#	include <stdalign.h>
#	ifndef constexpr
#		define constexpr const
#	endif
#	ifndef nullptr
#		define nullptr NULL
#	endif
#	ifndef static_assert
#		define static_assert(...) _Static_assert(__VA_ARGS__)
#	endif
#	ifndef alignas
#		define alignas _Alignas
#	endif
#	ifndef alignof
#		define alignof _Alignof
#	endif
#endif

#ifndef UINTPTR_C
#	if UINTPTR_WIDTH == 64
#		define UINTPTR_C(...) UINT64_C(__VA_ARGS__)
#	elif UINTPTR_WIDTH == 32
#		define UINTPTR_C(...) UINT32_C(__VA_ARGS__)
#	else
#		error "`UINTPTR_WIDTH` is neither 32 nor 64"
#	endif
#endif

#ifndef INTPTR_C
#	if INTPTR_WIDTH == 64
#		define INTPTR_C(...) INT64_C(__VA_ARGS__)
#	elif INTPTR_WIDTH == 32
#		define INTPTR_C(...) INT32_C(__VA_ARGS__)
#	else
#		error "`INTPTR_WIDTH` is neither 32 nor 64"
#	endif
#endif

#include <zerOS/platform.h>
#if zerOS_PLATFORM_IS_X86 || zerOS_PLATFORM_IS_AMD64
#	include <x86intrin.h>
#elif zerOS_PLATFORM_IS_ARM32 || zerOS_PLATFORM_IS_AARCH64
#	include <arm_acle.h>
#endif

typedef uint8_t __attribute__((__may_alias__)) zerOS_byte_t;
static_assert(sizeof(zerOS_byte_t) == 1);
static_assert(alignof(zerOS_byte_t) == 1);

#undef container_of
#define container_of(type, pointer, member) \
	((type*)((zerOS_byte_t*)pointer - offsetof(type, member)))

#ifndef auto
#	define auto __auto_type
#endif

#undef assert
#ifdef NDEBUG
#	define assert(expr)                                \
		((void)(0));                                    \
		({                                              \
			const bool UNIQUE(expr_in_assert) = (expr); \
			assume(UNIQUE(expr_in_assert));             \
		})
#else
#	define assert(expr)                                 \
		((void)sizeof((expr) ? 1 : 0), ({                \
			 const bool UNIQUE(expr_in_assert) = (expr); \
			 if (UNIQUE(expr_in_assert))                 \
				 assume(UNIQUE(expr_in_assert));         \
			 else                                        \
				 zerOS_hcf();                            \
		 }))
#endif

#undef assert_msg
#ifdef NDEBUG
#	define assert_msg(expr, ...)                       \
		((void)(0));                                    \
		({                                              \
			const bool UNIQUE(expr_in_assert) = (expr); \
			assume(UNIQUE(expr_in_assert));             \
		})
#else
#	define assert_msg(expr, ...)                        \
		((void)sizeof((expr) ? 1 : 0), ({                \
			 const bool UNIQUE(expr_in_assert) = (expr); \
			 if (UNIQUE(expr_in_assert))                 \
				 assume(UNIQUE(expr_in_assert));         \
			 else                                        \
				 zerOS_hcf();                            \
		 }))
#endif

#undef fast_log2_approx
#define fast_log2_approx(X) \
	((unsigned)(__CHAR_BIT__ * sizeof(typeof((X))) - __builtin_clzg((X), 0) - 1))

#undef is_power_of_two
#define is_power_of_two(value) ((bool)(value && !(value & (value - 1))))

#undef L1_CACHE_LINE_SIZE
#define L1_CACHE_LINE_SIZE 64

#undef PAGE_SIZE
#define PAGE_SIZE 4096

#undef MAX_ALIGN
#define MAX_ALIGN PAGE_SIZE

#undef MAX_ALIGN_LOG2
#define MAX_ALIGN_LOG2 PAGE_SIZE_LOG2

#undef MAX_VIRTUAL_ADDRESS_LOG2
#define MAX_VIRTUAL_ADDRESS_LOG2 57

#undef PAGE_SIZE_LOG2
#define PAGE_SIZE_LOG2 12

#undef likely
#undef unlikely
#undef expect
#define likely(...)         __builtin_expect((bool)(__VA_ARGS__), true)
#define unlikely(...)       __builtin_expect((bool)(__VA_ARGS__), false)
#define expect(expr, value) __builtin_expect((expr), (value))

#undef prefetch
#undef prefetch_ro
#undef prefetch_rw
#define prefetch(...)          __builtin_prefetch(__VA_ARGS__)
#define prefetch_ro(addr, ...) prefetch((addr), 0 __VA_OPT__(, __VA_ARGS__))
#define prefetch_rw(addr, ...) prefetch((addr), 1 __VA_OPT__(, __VA_ARGS__))

#undef prefetch_range
#undef prefetch_range_ro
#undef prefetch_range_rw
#define prefetch_range(addr, size, ...)                                                     \
	({                                                                                      \
		char* UNIQUE(addr_in_prefetch_range) = (char*)(addr);                               \
		for (size_t UNIQUE(i_in_prefetch_range)  = 0; UNIQUE(i_in_prefetch_range) < (size); \
			 UNIQUE(i_in_prefetch_range)        += L1_CACHE_LINE_SIZE)                      \
		{                                                                                   \
			prefetch(                                                                       \
			  UNIQUE(addr_in_prefetch_range)                                                \
			  + UNIQUE(i_in_prefetch_range) __VA_OPT__(, __VA_ARGS__));                     \
		}                                                                                   \
	})
#define prefetch_range_ro(addr, size, ...) \
	prefetch_range((addr), (size), 0 __VA_OPT__(, __VA_ARGS__))
#define prefetch_range_rw(addr, size, ...) \
	prefetch_range((addr), (size), 1 __VA_OPT__(, __VA_ARGS__))

#undef assume_aligned
#define assume_aligned(ptr, align) __builtin_assume_aligned((ptr), (align))

#undef assume
#define assume(cond) __builtin_assume((bool)(cond))

#undef assume_unaliasing_pointers
#define assume_unaliasing_pointers(ptr1, ptr2) __builtin_assume_separate_storage((ptr1), (ptr2))

#undef on_page_start
#define on_page_start(ptr) assume_aligned((ptr), (PAGE_SIZE))

#ifndef alloca
#	define alloca(size) __builtin_alloca(size)
#endif

#undef stack_alloc
#define stack_alloc(type) stack_alloc_array(type, 1)

#undef stack_alloc_array
#define stack_alloc_array(type, count) \
	__builtin_alloca_with_align((count) * sizeof(type), alignof(type))

#undef PP_PASTE
#define PP_PASTE(a, b) PP_PASTE_IMPL(a, b)

#undef PP_PASTE_IMPL
#define PP_PASTE_IMPL(a, b) a##b

#undef PP_PASTE4
#define PP_PASTE4(a, b, c, d) PP_PASTE4_IMPL(a, b, c, d)

#undef PP_PASTE4_IMPL
#define PP_PASTE4_IMPL(a, b, c, d) PP_PASTE(PP_PASTE(a, b), PP_PASTE(c, d))

#undef UNIQUE
#define UNIQUE(ident) PP_PASTE4(___uNiQuE_iDeNtIfIeR_at_LINE, __LINE__, _NAMED, ident)

#undef min
#undef max
#define min(a, b)                                                                  \
	({                                                                             \
		auto UNIQUE(a_in_min) = (a);                                               \
		auto UNIQUE(b_in_min) = (b);                                               \
		UNIQUE(a_in_min) < UNIQUE(b_in_min) ? UNIQUE(a_in_min) : UNIQUE(b_in_min); \
	})
#define max(a, b)                                                                  \
	({                                                                             \
		auto UNIQUE(a_in_max) = (a);                                               \
		auto UNIQUE(b_in_max) = (b);                                               \
		UNIQUE(a_in_max) > UNIQUE(b_in_max) ? UNIQUE(a_in_max) : UNIQUE(b_in_max); \
	})

#undef distance
#define distance(a, b)                                         \
	({                                                         \
		auto UNIQUE(a_in_distance) = (a);                      \
		auto UNIQUE(b_in_distance) = (b);                      \
		max(UNIQUE(a_in_distance), UNIQUE(b_in_distance))      \
		  - min(UNIQUE(a_in_distance), UNIQUE(b_in_distance)); \
	})

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

#undef zerOS_PP_FORCE_SEMICOLON
#define zerOS_PP_FORCE_SEMICOLON struct UNIQUE(__zerOS_semicolon_forcer)

#undef __zerOS_make_uX_impl
#define __zerOS_make_uX_impl(size)                                      \
	[[__gnu__::__always_inline__]]                                      \
	static inline PP_PASTE(uint, PP_PASTE(size, _t))                    \
	  PP_PASTE(zerOS_make_u, size)(uint8_t from, uint8_t to)            \
	{                                                                   \
		if (from < to && from < PP_PASTE(UINT, PP_PASTE(size, _WIDTH))) \
			return (PP_PASTE(UINT, PP_PASTE(size, _C))(1) << from)      \
				 | PP_PASTE(zerOS_make_u, size)(from + 1, to);          \
		return PP_PASTE(UINT, PP_PASTE(size, _C))(0);                   \
	}                                                                   \
	zerOS_PP_FORCE_SEMICOLON

__zerOS_make_uX_impl(8);
__zerOS_make_uX_impl(16);
__zerOS_make_uX_impl(32);
__zerOS_make_uX_impl(64);

[[__gnu__::__always_inline__]]
static inline uintptr_t zerOS_make_uptr(uint8_t from, uint8_t to)
{
	if (from < to && from < UINTPTR_WIDTH)
		return (UINTPTR_C(1) << from) | zerOS_make_uptr(from + 1, to);
	return UINTPTR_C(0);
}

#undef __zerOS_make_uX_impl

[[__gnu__::__always_inline__]]
static inline void zerOS_spin_hint(void)
{
#if zerOS_PLATFORM_IS_X86 || zerOS_PLATFORM_IS_AMD64
	_mm_pause();
#elif zerOS_PLATFORM_IS_ARM32
	// TODO: we need at least arm v6
	__yield();
#elif zerOS_PLATFORM_IS_AARCH64
	__isb(15); // ISB SY
#elif zerOS_PLATFORM_IS_RISCV32 || zerOS_PLATFORM_IS_RISCV64
	asm volatile("pause"
				 :
				 :
				 : "memory");
#endif
}

[[__noreturn__]]
static inline void zerOS_hcf(void)
{
	asm volatile("cli");
	while (true)
		zerOS_spin_hint();
}

#undef bit_at
#define bit_at(index, num)                                                      \
	({                                                                          \
		const auto UNIQUE(mask_in_bit_at) = ((typeof_unqual(num))1 << (index)); \
		(bool)(((num) & UNIQUE(mask_in_bit_at)) == UNIQUE(mask_in_bit_at));     \
	})

#endif
