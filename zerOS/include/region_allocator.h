#ifndef zerOS_REGION_ALLOCATOR_H_INCLUDED
#define zerOS_REGION_ALLOCATOR_H_INCLUDED

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

typedef uint8_t __attribute__((__may_alias__)) zerOS_byte_t;

enum zerOS_allocation_strategy
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

extern void* zerOS_region_allocator_alloc(
  struct zerOS_region_allocator* allocator,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy);
extern struct zerOS_region_allocator* zerOS_region_allocator_create(
  zerOS_byte_t*                  region,
  size_t                         region_size,
  bool                           static_mem,
  bool                           authorize_reclaim,
  enum zerOS_allocation_strategy preferred,
  zerOS_region_reclaim_hook_t    hook);

#endif
