#ifndef zerOS_REGION_ALLOCATOR_H_INCLUDED
#define zerOS_REGION_ALLOCATOR_H_INCLUDED

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __INTELLISENSE__
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

enum zerOS_allocation_strategy
#ifndef __INTELLISENSE__
  : uint8_t
#endif
{
	zerOS_ALLOC_STRAT_DEFAULT,
	zerOS_ALLOC_STRAT_BEST_FIT,
	zerOS_ALLOC_STRAT_FIRST_FIT,
};

typedef bool (*zerOS_region_reclaim_hook_t)(
  zerOS_byte_t*, /* region */
  size_t,        /* region_page_count */
  bool           /* is_static_memory */
);

struct zerOS_region_allocator;

extern struct zerOS_region_allocator* zerOS_region_allocator_create(
  zerOS_byte_t*                  region,
  size_t                         region_size,
  bool                           static_mem,
  bool                           authorize_reclaim,
  enum zerOS_allocation_strategy preferred,
  zerOS_region_reclaim_hook_t    hook);

extern void* zerOS_region_allocator_alloc(
  struct zerOS_region_allocator* allocator,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy);

extern void* zerOS_region_allocator_realloc(
  struct zerOS_region_allocator* allocator,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy);

extern void zerOS_region_allocator_dealloc(struct zerOS_region_allocator* allocator, void* ptr);

#endif
