#ifndef zerOS_REGION_ALLOCATOR_H_INCLUDED_
#define zerOS_REGION_ALLOCATOR_H_INCLUDED_ 1

#include <zerOS/common.h>

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
  void*                          ptr,
  size_t                         old_size,
  size_t                         old_align,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy);

extern void zerOS_region_allocator_dealloc(struct zerOS_region_allocator* allocator, void* ptr);

extern bool zerOS_region_allocator_contains(struct zerOS_region_allocator* allocator, void* ptr);

extern bool zerOS_region_allocator_is_static_region(struct zerOS_region_allocator* allocator);

extern bool zerOS_region_allocator_reclaim(struct zerOS_region_allocator* allocator);

extern size_t
zerOS_region_allocator_max_size_for(struct zerOS_region_allocator* allocator, void* ptr);

#if 0
extern struct zerOS_region_allocator*
zerOS_region_allocator_prev(struct zerOS_region_allocator* allocator);

extern struct zerOS_region_allocator*
zerOS_region_allocator_next(struct zerOS_region_allocator* allocator);

extern void zerOS_region_allocator_set_prev(
  struct zerOS_region_allocator* allocator, struct zerOS_region_allocator* prev);

extern void zerOS_region_allocator_set_next(
  struct zerOS_region_allocator* allocator, struct zerOS_region_allocator* next);
#endif

struct zerOS_region_allocator_additional_space_info
{
	zerOS_byte_t* addr;
	size_t        size;
};

extern struct zerOS_region_allocator_additional_space_info
zerOS_region_allocator_additional_space(struct zerOS_region_allocator* allocator);

#endif
