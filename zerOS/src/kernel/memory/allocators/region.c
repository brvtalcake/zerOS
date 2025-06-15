// TODO: libdivide is only available for x86-64 targets...
// TODO: random idea I had when writing code here: write a clang plugin to optimize data structure
//       layout for structure marked with a custom attribute

#include <stdalign.h>
#include <stdatomic.h>

#include <zerOS/address.h>
#include <zerOS/common.h>
#include <zerOS/guard.h>
#include <zerOS/platform.h>
#include <zerOS/rbtree.h>
#include <zerOS/region_allocator.h>
#include <zerOS/spinlock.h>

#undef HAVE_LIBDIVIDE
#if zerOS_PLATFORM_IS_X86    \
  || zerOS_PLATFORM_IS_AMD64 \
  || zerOS_PLATFORM_IS_ARM32 \
  || zerOS_PLATFORM_IS_AARCH64
#	define HAVE_LIBDIVIDE 1
#	include <libdivide.h>
#else
#	define HAVE_LIBDIVIDE 0
#endif

struct subregion_node
{
	// this remains in memory even when allocated, hence it shall never be overwritten by user
	struct [[gnu::packed]] subregion_node_persistent
	{
		// since it's aligned on page start, we can further shrink the xored addresses.
		// only here to be able to coalesce free pages
		uintptr_t xored_prev_next : (MAX_VIRTUAL_ADDRESS_LOG2 - PAGE_SIZE_LOG2);
		bool      free : 1;
		size_t    page_count : (MAX_VIRTUAL_ADDRESS_LOG2 - PAGE_SIZE_LOG2);
	} persistent;

	// the following structures can be overwritten by user when it has been allocated, therefore it
	// is not marked with `[[gnu::packed]]`, since it won't take too much user space on user
	// allocated pages

#if zerOS_REGION_ALLOCATOR_USE_FREELIST
	// this structure is used for fast O(1) (?) first-fit allocations
	// TODO: maybe sort them by size, bigest ones first ?
	struct subregion_node_free_list_info
	{
		struct subregion_node* prev;
		struct subregion_node* next;
	} free_list_info;
#endif

	// and this one is for O(log(N)) best-fit allocations
	struct subregion_node_rbtree_info
	{
#if 0
		union subregion_node_bucket
#else
		union
#endif
		{
			struct subregion_node_bucket_head
			{
				alignas(max_align_t) struct zerOS_rbtree_head head;
				struct subregion_node_bucket_list
				{
					struct subregion_node* head;
					struct subregion_node* tail;
				} list;
			} as_head;
			struct subregion_node_bucket_member
			{
				struct subregion_node_bucket_head* bucket_head;
				struct subregion_node*             prev;
				struct subregion_node*             next;
			} as_member;
		};
		bool is_bucket_head;
	} rbtree_info;
};

[[__gnu__::__pure__]]
static inline size_t subregion_page_count(const struct subregion_node* restrict const node)
{
	return node->persistent.page_count;
}

static inline void subregion_set_page_count(struct subregion_node* node, size_t new_count)
{
	node->persistent.page_count = new_count;
}

[[__gnu__::__pure__]]
static inline size_t subregion_size(const struct subregion_node* restrict const node)
{
	return subregion_page_count(node) * PAGE_SIZE;
}

static inline void subregion_set_size(struct subregion_node* node, size_t new_count)
{
	node->persistent.page_count = new_count / PAGE_SIZE;
}

[[__gnu__::__pure__]]
static inline bool subregion_free(const struct subregion_node* restrict const node)
{
	return node->persistent.free;
}

static inline void subregion_set_free(struct subregion_node* node, bool new_value)
{
	node->persistent.free = new_value;
}

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node* node_at(zerOS_byte_t* addr)
{
	return on_page_start((struct subregion_node*)addr);
}

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node* node_at(uintptr_t addr)
{
	return node_at((zerOS_byte_t*)addr);
}

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node* node_at(void* addr)
{
	return node_at((zerOS_byte_t*)addr);
}

enum offset_kind
{
	OFFSET_KIND_PAGE,
	OFFSET_KIND_RAW
};

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node*
node_at(zerOS_byte_t* base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	const size_t multiplier = offkind == OFFSET_KIND_PAGE ? PAGE_SIZE : 1;
	return node_at(base_addr + (offset * multiplier));
}

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node*
node_at(uintptr_t base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	return node_at((zerOS_byte_t*)base_addr, offset, offkind);
}

[[clang::overloadable]] [[maybe_unused]]
static inline struct subregion_node*
node_at(void* base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	return node_at((zerOS_byte_t*)base_addr, offset, offkind);
}

#if zerOS_REGION_ALLOCATOR_USE_FREELIST

struct subregion_free_list
{
	struct subregion_node* head;
	struct subregion_node* tail;
};

static inline void
subregion_free_list_add(struct subregion_free_list* list, struct subregion_node* node)
{
	node->free_list_info.next = nullptr;
	if (unlikely(!list->head && !list->tail))
	{
		node->free_list_info.prev = nullptr;
		list->head                = node;
	}
	else
	{
		// put the node at the end
		list->tail->free_list_info.next = node;
		node->free_list_info.prev       = list->tail;
	}
	list->tail = node;
}

static inline void
subregion_free_list_remove(struct subregion_free_list* list, struct subregion_node* node)
{
	if (node->free_list_info.prev)
		node->free_list_info.prev->free_list_info.next = node->free_list_info.next;
	else
		list->head = node->free_list_info.next;

	if (node->free_list_info.next)
		node->free_list_info.next->free_list_info.prev = node->free_list_info.prev;
	else
		list->tail = node->free_list_info.prev;
}

static inline struct subregion_node*
subregion_free_list_find_first(struct subregion_free_list* list, const size_t min_page_count)
{
	struct subregion_node* iter = list->head;
	do
		if (subregion_page_count(iter) >= min_page_count)
			break;
	while ((iter = iter->free_list_info.next));
	return iter;
}

static inline struct subregion_free_list subregion_free_list_new(struct subregion_node* node)
{
	struct subregion_free_list ret = { .head = nullptr, .tail = nullptr };
	subregion_free_list_add(&ret, node);
	return ret;
}

#endif

struct subregion_list
{
	struct subregion_node* head;
	struct subregion_node* tail;
};

[[__gnu__::__pure__]]
static inline struct subregion_node* subregion_list_prev(
  const struct subregion_node* restrict const node,
  const struct subregion_node* restrict const next)
{
	const uintptr_t zeroext_xored = zerOS_virtaddr_zero_extend(node->persistent.xored_prev_next);
	const uintptr_t zeroext_next  = zerOS_virtaddr_zero_extend((uintptr_t)next) >> PAGE_SIZE_LOG2;
	const uintptr_t zeroext_prev  = (zeroext_xored ^ zeroext_next) << PAGE_SIZE_LOG2;

	return node_at(zerOS_virtaddr_canonicalize(zeroext_prev));
}

[[__gnu__::__pure__]]
static inline struct subregion_node* subregion_list_next(
  const struct subregion_node* restrict const node,
  const struct subregion_node* restrict const prev)
{
	const uintptr_t zeroext_xored = zerOS_virtaddr_zero_extend(node->persistent.xored_prev_next);
	const uintptr_t zeroext_prev  = zerOS_virtaddr_zero_extend((uintptr_t)prev) >> PAGE_SIZE_LOG2;
	const uintptr_t zeroext_next  = (zeroext_xored ^ zeroext_prev) << PAGE_SIZE_LOG2;

	return node_at(zerOS_virtaddr_canonicalize(zeroext_next));
}

static inline void subregion_list_set_prev_next(
  struct subregion_node* node, struct subregion_node* prev, struct subregion_node* next)
{
	const uintptr_t zeroext_prev = zerOS_virtaddr_zero_extend((uintptr_t)prev) >> PAGE_SIZE_LOG2;
	const uintptr_t zeroext_next = zerOS_virtaddr_zero_extend((uintptr_t)next) >> PAGE_SIZE_LOG2;

	node->persistent.xored_prev_next = zeroext_prev ^ zeroext_next;
}

static inline void subregion_list_new(struct subregion_list* list, struct subregion_node* node)
{
	subregion_list_set_prev_next(node, nullptr, nullptr);
	list->head = node;
	list->tail = node;
}

[[maybe_unused]]
static inline void subregion_list_insert_before(
  struct subregion_list* restrict list, struct subregion_node* at, struct subregion_node* new_node)
{
	assert(list);
	assert(at);
	assert(new_node);
	assert(list->head);
	assert(list->tail);

	enum
	{
		LIST_PREV = 0,
		LIST_CURR,
		LIST_NEXT
	};

	struct subregion_node* const tmp =
	  at == list->tail ? nullptr
					   : node_at((uintptr_t)at, subregion_page_count(at), OFFSET_KIND_PAGE);
	struct subregion_node* const current_state[3] = {
		[LIST_CURR] = at, [LIST_NEXT] = tmp, [LIST_PREV] = subregion_list_prev(at, tmp)
	};

	// set the "next" of `at`'s current "prev" as being `new_node`
	if (current_state[LIST_PREV])
	{
		struct subregion_node* const prev_prev =
		  subregion_list_prev(current_state[LIST_PREV], current_state[LIST_CURR]);
		subregion_list_set_prev_next(current_state[LIST_PREV], prev_prev, new_node);
	}
	else
		// as of now, we should have: `list->head == at`
		list->head = new_node;

	subregion_list_set_prev_next(new_node, current_state[LIST_PREV], current_state[LIST_CURR]);

	subregion_list_set_prev_next(at, new_node, current_state[LIST_NEXT]);
}

static inline void subregion_list_insert_after(
  struct subregion_list* restrict list, struct subregion_node* at, struct subregion_node* new_node)
{
	assert(list);
	assert(at);
	assert(new_node);
	assert(list->head);
	assert(list->tail);

	enum
	{
		LIST_PREV = 0,
		LIST_CURR,
		LIST_NEXT
	};

	struct subregion_node* const tmp =
	  at == list->tail ? nullptr
					   : node_at((uintptr_t)at, subregion_page_count(at), OFFSET_KIND_PAGE);
	struct subregion_node* const current_state[3] = {
		[LIST_CURR] = at, [LIST_NEXT] = tmp, [LIST_PREV] = subregion_list_prev(at, tmp)
	};

	// set the "next" of `at`'s current "prev" as being `new_node`
	if (current_state[LIST_NEXT])
	{
		struct subregion_node* const next_next =
		  subregion_list_next(current_state[LIST_NEXT], current_state[LIST_CURR]);
		subregion_list_set_prev_next(current_state[LIST_NEXT], new_node, next_next);
	}
	else
		// as of now, we should have: `list->tail == at`
		list->tail = new_node;

	subregion_list_set_prev_next(new_node, current_state[LIST_CURR], current_state[LIST_NEXT]);

	subregion_list_set_prev_next(at, current_state[LIST_PREV], new_node);
}

static inline void subregion_list_delete(
  struct subregion_list* restrict list,
  struct subregion_node* deleted,
  struct subregion_node* prev,
  struct subregion_node* next)
{
	assert(list);
	assert(list->head);
	assert(list->tail);
	assert(deleted);

	if (prev)
	{
		struct subregion_node* prev_prev = subregion_list_prev(prev, deleted);
		subregion_list_set_prev_next(prev, prev_prev, next);
	}
	else
		list->head = next;

	if (next)
	{
		struct subregion_node* next_next = subregion_list_next(next, deleted);
		subregion_list_set_prev_next(next, prev, next_next);
	}
	else
		list->tail = prev;
}

static inline struct subregion_node* subregion_node_from_rbtree_head(struct zerOS_rbtree_head* head)
{
	static_assert(offsetof(struct subregion_node_bucket_head, head) == 0);
	struct subregion_node_bucket_head* bucket_head =
	  zerOS_rbtree_head_to_container(struct subregion_node_bucket_head, head);
	struct subregion_node_rbtree_info* rbtree_info =
	  container_of(struct subregion_node_rbtree_info, bucket_head, as_head);
	struct subregion_node* node = container_of(struct subregion_node, rbtree_info, rbtree_info);
	return on_page_start(node);
}

[[maybe_unused]]
static int subregion_rbtree_compare_nodes(struct zerOS_rbtree_head* head, void* data)
{
	const struct subregion_node* lhs_ptr        = subregion_node_from_rbtree_head(head);
	const struct subregion_node* rhs_ptr        = on_page_start(data);
	const size_t                 lhs_page_count = subregion_page_count(lhs_ptr);
	const size_t                 rhs_page_count = subregion_page_count(rhs_ptr);
	if (lhs_page_count < rhs_page_count)
		return -1;
	else if (lhs_page_count > rhs_page_count)
		return 1;
	else if (lhs_page_count == rhs_page_count)
		return 0;
	unreachable();
}

static int subregion_rbtree_compare_page_count(struct zerOS_rbtree_head* head, void* data)
{
	const struct subregion_node* lhs_ptr        = subregion_node_from_rbtree_head(head);
	const size_t*                rhs_ptr        = data;
	const size_t                 lhs_page_count = subregion_page_count(lhs_ptr);
	const size_t                 rhs_page_count = *rhs_ptr;
	if (lhs_page_count < rhs_page_count)
		return -1;
	else if (lhs_page_count > rhs_page_count)
		return 1;
	else if (lhs_page_count == rhs_page_count)
		return 0;
	unreachable();
}

/**
 * @brief Insert a node in a pre-existing bucket.
 * @post The bucket has at least 2 elements, i.e. the list is not empty
 * @param head_node A bucket head
 * @param node The node to insert in the bucket
 * @note It does not operate on the containing rbtree.
 */
static inline void subregion_rbtree_add_to_bucket(
  struct subregion_node* const head_node, struct subregion_node* restrict const node)
{
	assume_unaliasing_pointers(head_node, node);

	struct subregion_node_bucket_head* bucket = &head_node->rbtree_info.as_head;

	struct subregion_node* curr_head = bucket->list.head;
	struct subregion_node* curr_tail = bucket->list.tail;

	if (!curr_head && !curr_tail)
		// one-element bucket, empty list
		bucket->list.head = node;
	else
		// at least two element in the bucket, at least one in the list
		curr_tail->rbtree_info.as_member.next = node;
	bucket->list.tail = node;

	node->rbtree_info.is_bucket_head        = false;
	node->rbtree_info.as_member.bucket_head = bucket;
	node->rbtree_info.as_member.prev        = curr_tail;
	node->rbtree_info.as_member.next        = nullptr;
}

/**
 * @brief Get a node from the specified bucket head.
 * @pre The bucket must contains at least 2 elements, i.e. the list is not empty.
 * @param head_node A bucket head
 * @return A node from the bucket
 * @note It does not operate on the containing rbtree.
 */
static inline struct subregion_node*
subregion_rbtree_take_from_bucket(struct subregion_node* const head_node)
{
	assert(head_node);

	struct subregion_node_bucket_head* bucket = &head_node->rbtree_info.as_head;

	struct subregion_node* restrict const returned = bucket->list.head;

	bucket->list.head = returned->rbtree_info.as_member.next;
	if (bucket->list.head)
		bucket->list.head->rbtree_info.as_member.prev = nullptr;
	else
		bucket->list.tail = nullptr;

	return returned;
}

static inline void subregion_rbtree_delete(struct subregion_node* const node)
{
	assert(node);

	if (node->rbtree_info.is_bucket_head)
	{
		if (!node->rbtree_info.as_head.list.head && !node->rbtree_info.as_head.list.tail)
			zerOS_rbtree_unlink_and_rebalance(&node->rbtree_info.as_head.head);
		else
		{
			struct subregion_node* new_bucket_head = subregion_rbtree_take_from_bucket(node);
			new_bucket_head->rbtree_info           = node->rbtree_info;
			zerOS_rbtree_replace(
			  &node->rbtree_info.as_head.head, &new_bucket_head->rbtree_info.as_head.head);
		}
	}
	else
	{
		struct subregion_node* bucket_prev = node->rbtree_info.as_member.prev;
		struct subregion_node* bucket_next = node->rbtree_info.as_member.next;

		if (bucket_prev)
			bucket_prev->rbtree_info.as_member.next = bucket_next;
		else
			node->rbtree_info.as_member.bucket_head->list.head = bucket_next;

		if (bucket_next)
			bucket_next->rbtree_info.as_member.prev = bucket_prev;
		else
			node->rbtree_info.as_member.bucket_head->list.tail = bucket_prev;
	}
}

static inline void subregion_rbtree_insert(
  struct zerOS_rbtree* restrict const rbtree, struct subregion_node* const node)
{
	zerOS_rbtree_reset_head(&node->rbtree_info.as_head.head);

	size_t page_count = subregion_page_count(node);

	struct zerOS_rbtree_head* found = zerOS_rbtree_insert(
	  rbtree, &node->rbtree_info.as_head.head, &subregion_rbtree_compare_page_count, &page_count);

	if (found)
	{
		struct subregion_node* found_node = subregion_node_from_rbtree_head(found);
		subregion_rbtree_add_to_bucket(found_node, node);
	}
	else
	{
		node->rbtree_info.is_bucket_head    = true;
		node->rbtree_info.as_head.list.head = nullptr;
		node->rbtree_info.as_head.list.tail = nullptr;
	}
}

// # Precondition
// The node is a bucket head
static inline struct subregion_node*
subregion_rbtree_take_from_bucket_or_unlink(struct zerOS_rbtree_head* const rbhead)
{
	struct subregion_node* const head_node = subregion_node_from_rbtree_head(rbhead);
	if (!head_node->rbtree_info.as_head.list.head && !head_node->rbtree_info.as_head.list.tail)
	{
		zerOS_rbtree_unlink_and_rebalance(&head_node->rbtree_info.as_head.head);
		return head_node;
	}
	else
		return subregion_rbtree_take_from_bucket(head_node);
}

static inline struct zerOS_rbtree_head* subregion_rbtree_extract_best_lower_bounded_helper(
  struct zerOS_rbtree_head* head, const size_t min_page_count)
{
	while (head && subregion_page_count(subregion_node_from_rbtree_head(head)) < min_page_count)
		head = zerOS_rbtree_get_prevnext(head, zerOS_RBTREE_RIGHT);
	return head;
}

static inline struct subregion_node* subregion_rbtree_extract_best_lower_bounded(
  struct zerOS_rbtree* restrict const rbtree, const size_t min_page_count)
{
	struct zerOS_rbtree_head*   parent;
	enum zerOS_rbtree_direction dir;
	size_t                      data = min_page_count;
	struct zerOS_rbtree_head*   perfect_match =
	  zerOS_rbtree_find_parent(rbtree, &subregion_rbtree_compare_page_count, &data, &parent, &dir);
	return subregion_rbtree_take_from_bucket_or_unlink(
	  perfect_match
		?: (dir == zerOS_RBTREE_LEFT
			  ? parent
			  : subregion_rbtree_extract_best_lower_bounded_helper(parent, min_page_count)));
}

static inline struct subregion_node* subregion_rbtree_extract_first_lower_bounded(
  struct zerOS_rbtree* restrict const rbtree, const size_t min_page_count)
{
	struct zerOS_rbtree_head* head = zerOS_rbtree_get_base(rbtree);
	while (head && subregion_page_count(subregion_node_from_rbtree_head(head)) < min_page_count)
		head = zerOS_rbtree_get_child(head, zerOS_RBTREE_RIGHT);
	return head ? subregion_rbtree_take_from_bucket_or_unlink(head) : nullptr;
}

static inline void subregion_rbtree_new(struct zerOS_rbtree* rbtree, struct subregion_node* node)
{
	zerOS_rbtree_reset_tree(rbtree);
	subregion_rbtree_insert(rbtree, node);
}

struct zerOS_region_allocator
{
	struct subregion_list list;
	struct zerOS_rbtree   rbtree;
#if zerOS_REGION_ALLOCATOR_USE_FREELIST
	struct subregion_free_list free_list;
#endif

	zerOS_byte_t* region;
	size_t        region_page_count;

	enum zerOS_allocation_strategy preferred_strategy;
	bool                           is_static_memory;
	bool                           authorize_reclaim;
	zerOS_region_reclaim_hook_t    reclaim_hook;

	alignas(64) struct zerOS_spinlock spinlock;
};

static_assert(sizeof(struct zerOS_region_allocator) < PAGE_SIZE);

/**
 * @brief The size of a `struct subregion_node` when not allocated
 */
[[maybe_unused]]
static constexpr unsigned MEMNODE_SIZE = sizeof(struct subregion_node);
/**
 * @brief The size of a `struct subregion_node` when allocated
 */
[[maybe_unused]]
static constexpr unsigned MEMNODE_ALLOCATED_SIZE = sizeof(struct subregion_node_persistent);

static inline void subregion_split(
  struct zerOS_region_allocator* manager,
  struct subregion_node*         node,
  ptrdiff_t                      new_page_offset,
  size_t                         new_page_count)
{
	struct subregion_node* new_node = node_at((uintptr_t)node, new_page_offset, OFFSET_KIND_PAGE);

	subregion_set_page_count(new_node, new_page_count);
	subregion_set_free(new_node, true);

	if (new_page_offset >= 0)
		subregion_list_insert_after(&manager->list, node, new_node);
	else
		subregion_list_insert_before(&manager->list, node, new_node);

	subregion_rbtree_insert(&manager->rbtree, new_node);
}

union fast_divmod
{
	struct fast_divmod_input
	{
		size_t numer;
		size_t denom;
	} in;
	struct fast_divmod_output
	{
		size_t div;
		size_t mod;
	} out;
};

#if HAVE_LIBDIVIDE

struct libdivide_wrapper
{
#	if SIZE_WIDTH == 32
	struct libdivide_u32_branchfree_t divider;
#	elif SIZE_WIDTH == 64
	struct libdivide_u64_branchfree_t divider;
#	else
#		error "unsupported SIZE_WIDTH"
#	endif
	bool initialized;
};

static inline void libdivide_wrapper_init(struct libdivide_wrapper* wrapper, size_t denom)
{
	if (wrapper->initialized)
		return;

#	if SIZE_WIDTH == 32
	wrapper->divider = libdivide_u32_branchfree_gen(denom);
#	elif SIZE_WIDTH == 64
	wrapper->divider = libdivide_u64_branchfree_gen(denom);
#	endif
	wrapper->initialized = true;
}

static inline size_t
libdivide_wrapper_calc(struct libdivide_wrapper* wrapper, size_t numer, size_t denom)
{
	if (!wrapper->initialized)
		libdivide_wrapper_init(wrapper, denom);

#	if SIZE_WIDTH == 32
	return libdivide_u32_branchfree_do(numer, &wrapper->divider);
#	elif SIZE_WIDTH == 64
	return libdivide_u64_branchfree_do(numer, &wrapper->divider);
#	endif
}
#endif

[[__maybe_unused__]]
static void fast_divmod(union fast_divmod* inout)
{
#if HAVE_LIBDIVIDE
	static struct zerOS_spinlock spinlock = zerOS_SPINLOCK_INITIALIAZER;

	// clang-format off
	// NOTE: `fast_dividers[0]` corresponds to the divider for
	//		 `denom == 1`
	static struct libdivide_wrapper fast_dividers[(MAX_ALIGN - 1)] = {
		[0 ... (MAX_ALIGN - 1) - 1] = { .initialized = false }
	};
	// clang-format on

	const size_t numer = inout->in.numer;
	const size_t denom = inout->in.denom;

	zerOS_spin_lock(&spinlock);

	struct libdivide_wrapper* divider = fast_dividers + denom - 1;

	inout->out.div = libdivide_wrapper_calc(divider, numer, denom);

	zerOS_spin_unlock(&spinlock);

	inout->out.mod = numer - (inout->out.div * denom);
#else
	const size_t numer = inout->in.numer;
	const size_t denom = inout->in.denom;

	inout->out.div = numer / denom;
	inout->out.mod = numer % denom;
#endif
}

[[__maybe_unused__]]
static void fast_divmod_by_power_of_two(union fast_divmod* inout)
{
	const size_t numer = inout->in.numer;
	const size_t denom = inout->in.denom;

	size_t log2 = fast_log2_approx(denom);

	inout->out.div = numer >> log2;
	inout->out.mod = numer & ((1ull << log2) - 1);
}

// TODO: align_up and align_down macros or functions

/**
 * @brief Align a pointer up
 * @param ptr The pointer
 * @param align The alignment
 * @returns The aligned pointer value
 * @pre `align` must be a power of two
 */
static inline uintptr_t align_up(uintptr_t ptr, size_t align)
{
	assert(align % 2 == 0 || align == 1);
	assert(is_power_of_two(align));

	static_assert(sizeof(uintptr_t) == sizeof(size_t));
	static_assert(alignof(uintptr_t) == alignof(size_t));

	union fast_divmod divmod = {
		.in = { .numer = ptr, .denom = align }
	};
	fast_divmod_by_power_of_two(&divmod);
	return ptr + (align - divmod.out.mod);
}

static inline uintptr_t find_header(uintptr_t user_ptr)
{
	return (user_ptr - 1) - ((user_ptr - 1) % PAGE_SIZE);
}

static inline bool region_init_sanity_checks(uintptr_t addr, size_t size)
{
	return (addr % PAGE_SIZE == 0) && (size % PAGE_SIZE == 0) && (size > 2 * PAGE_SIZE);
}

extern struct zerOS_region_allocator* zerOS_region_allocator_create(
  zerOS_byte_t*                  region,
  size_t                         region_size,
  bool                           static_mem,
  bool                           authorize_reclaim,
  enum zerOS_allocation_strategy preferred,
  zerOS_region_reclaim_hook_t    hook)
{
	if (!region_init_sanity_checks((uintptr_t)region, region_size))
		return nullptr;

	struct zerOS_region_allocator* ret = on_page_start(region);

	ret->spinlock = zerOS_SPINLOCK_INITIALIAZER;
	if (!zerOS_spin_try_lock(&ret->spinlock))
		return nullptr;

	struct subregion_node* first_node = on_page_start(region + PAGE_SIZE);

	subregion_set_size(first_node, region_size - PAGE_SIZE);
	subregion_set_free(first_node, true);

	subregion_list_new(&ret->list, first_node);
	subregion_rbtree_new(&ret->rbtree, first_node);

#if zerOS_REGION_ALLOCATOR_USE_FREELIST
	ret->free_list = subregion_free_list_new(first_node);
#endif

	ret->region            = region;
	ret->region_page_count = region_size / PAGE_SIZE;

	ret->is_static_memory = static_mem;
	ret->preferred_strategy =
	  preferred != zerOS_ALLOC_STRAT_DEFAULT ? preferred : zerOS_ALLOC_STRAT_BEST_FIT;

	if (authorize_reclaim && hook)
	{
		ret->authorize_reclaim = true;
		ret->reclaim_hook      = hook;
	}
	else
	{
		ret->authorize_reclaim = false;
		ret->reclaim_hook      = nullptr;
	}

#if 0
	zerOS_region_allocator_set_prev(ret, nullptr);
	zerOS_region_allocator_set_next(ret, nullptr);
#endif

	zerOS_spin_unlock(&ret->spinlock);

	return ret;
}

[[__gnu__::__const__]]
static inline size_t padding_for(size_t align)
{
	assert(is_power_of_two(align));

	const uintptr_t aligned = align_up(MEMNODE_ALLOCATED_SIZE, align);
	return aligned - MEMNODE_ALLOCATED_SIZE;
}

static void*
region_alloc_best_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	if (zerOS_rbtree_is_empty(&allocator->rbtree))
		return nullptr;

	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum  = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;
	const size_t absolute_page_count_minimum =
	  (absolute_minimum / PAGE_SIZE) + (absolute_minimum % PAGE_SIZE ? 1 : 0);

	struct subregion_node* node =
	  subregion_rbtree_extract_best_lower_bounded(&allocator->rbtree, absolute_page_count_minimum);
	if (node)
	{
		// we found a chunk
		const size_t unused_pages = subregion_page_count(node) - absolute_page_count_minimum;

		subregion_set_free(node, false);

		if (unused_pages != 0)
		{
			subregion_set_page_count(node, absolute_page_count_minimum);
			subregion_split(allocator, node, absolute_page_count_minimum, unused_pages);
		}

		return (zerOS_byte_t*)node + MEMNODE_ALLOCATED_SIZE + alignment_padding;
	}

	// failed to get a suitable chunk
	return nullptr;
}

static void*
region_alloc_first_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	if (zerOS_rbtree_is_empty(&allocator->rbtree))
		return nullptr;

	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum  = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;
	const size_t absolute_page_count_minimum =
	  (absolute_minimum / PAGE_SIZE) + (absolute_minimum % PAGE_SIZE ? 1 : 0);

	struct subregion_node* node =
	  subregion_rbtree_extract_first_lower_bounded(&allocator->rbtree, absolute_page_count_minimum);

	if (node)
	{
		// we found a chunk
		const size_t unused_pages = subregion_page_count(node) - absolute_page_count_minimum;

		subregion_set_free(node, false);

		if (unused_pages != 0)
		{
			subregion_set_page_count(node, absolute_page_count_minimum);
			subregion_split(allocator, node, absolute_page_count_minimum, unused_pages);
		}

		return (zerOS_byte_t*)node + MEMNODE_ALLOCATED_SIZE + alignment_padding;
	}

	return nullptr;
}

static inline bool layout_requirements_ok(size_t size, size_t align)
{
	return likely(align <= MAX_ALIGN) && likely(is_power_of_two(align)) && likely(size);
}

extern void* zerOS_region_allocator_alloc(
  struct zerOS_region_allocator* allocator,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy)
{
	assert(allocator->list.head);
	assert(allocator->list.tail);

	if (unlikely(align == SIZE_MAX))
		align = alignof(max_align_t);

	if (!layout_requirements_ok(size, align))
		return nullptr;

	prefetch_rw(allocator, 2);

	void* returned = nullptr;

	zerOS_spin_lock(&allocator->spinlock);

	enum zerOS_allocation_strategy strat =
	  strategy != zerOS_ALLOC_STRAT_DEFAULT ? strategy : allocator->preferred_strategy;
	assert(strat == zerOS_ALLOC_STRAT_BEST_FIT || strat == zerOS_ALLOC_STRAT_FIRST_FIT);

	switch (strat)
	{
		case zerOS_ALLOC_STRAT_BEST_FIT:
			returned = region_alloc_best_fit(allocator, size, align);
			break;
		case zerOS_ALLOC_STRAT_FIRST_FIT:
			returned = region_alloc_first_fit(allocator, size, align);
			break;
		default:
			unreachable();
			break;
	}

	zerOS_spin_unlock(&allocator->spinlock);

	assert((uintptr_t)returned % align == 0);
	assert((uintptr_t)returned > (uintptr_t)allocator->region + PAGE_SIZE);
	assert(
	  (uintptr_t)returned
	  < (uintptr_t)allocator->region + allocator->region_page_count * PAGE_SIZE);

	return returned;
}

extern bool zerOS_region_allocator_contains(struct zerOS_region_allocator* allocator, void* ptr)
{
	if (unlikely(!ptr))
		return true;
	return (uintptr_t)ptr >= (uintptr_t)allocator->region
		&& (uintptr_t)ptr
			 <= (uintptr_t)allocator->region + (allocator->region_page_count * PAGE_SIZE);
}

extern void zerOS_region_allocator_dealloc(struct zerOS_region_allocator* allocator, void* ptr)
{
	prefetch_rw(allocator, 2);

	struct subregion_node* const node = node_at(find_header((uintptr_t)ptr));
	struct subregion_node* const next =
	  node == allocator->list.tail
		? nullptr
		: node_at((uintptr_t)node, subregion_page_count(node), OFFSET_KIND_PAGE);
	struct subregion_node* const prev = subregion_list_prev(node, next);

	struct subregion_node* node_to_insert = node;
	size_t                 new_size       = subregion_page_count(node);

	zerOS_spin_lock(&allocator->spinlock);

	/*
	 * Coalesce the nearest free chunks together. The "leftmost" chunk is the one that will remain
	 * at the end of the "coalescencing" operation, so we distinguish two cases :
	 *   - the case where `prev` is the chunk remaining
	 *   - the case where `node` itself is the chunk remaining
	 * For now, node that are free before `zerOS_region_allocator_dealloc` are deleted from the free
	 * node rbtree before coalescing. Only the remaining node is then put back in the tree.
	 * Also note that even if we call `subregion_list_delete`, the storage for the header is still
	 * there and (most importantly) untouched.
	 */
	if (prev && subregion_free(prev))
	{
		/*
		 * Since `prev` is freed, then it is in the red-black tree. We thus have to temporarily
		 * remove it from the tree in order to make modifications.
		 */
		subregion_rbtree_delete(prev);
		node_to_insert  = prev;
		new_size       += subregion_page_count(prev);

		// in the node list, we have:
		//     ... <-> prev <-> node <-> next <-> ...

		subregion_list_delete(&allocator->list, node, prev, next);
		// now we have:
		//     ... <-> prev <-> next <->  ...

		if (next && subregion_free(next))
		{
			subregion_rbtree_delete(next);
			new_size += subregion_page_count(next);

			struct subregion_node* next_next = subregion_list_next(next, node);
			subregion_list_delete(&allocator->list, next, prev, next_next);
			// and finally we have:
			//     ... <-> prev <-> next_next <-> ...
		}
	}
	else
	{
		// in the node list, we have:
		//     ... <-> node <-> next <-> ...

		subregion_set_free(node, true);
		if (next && subregion_free(next))
		{
			subregion_rbtree_delete(next);
			new_size += subregion_page_count(next);

			struct subregion_node* next_next = subregion_list_next(next, node);
			subregion_list_delete(&allocator->list, next, node, next_next);
		}
	}

	subregion_set_page_count(node_to_insert, new_size);
	subregion_rbtree_insert(&allocator->rbtree, node_to_insert);

	zerOS_spin_unlock(&allocator->spinlock);
}

extern void* zerOS_region_allocator_realloc(
  struct zerOS_region_allocator* allocator,
  void*                          ptr,
  size_t                         old_size,
  size_t                         old_align,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy)
{
	prefetch_rw(allocator, 2);

	if (unlikely(!ptr))
		return zerOS_region_allocator_alloc(allocator, size, align, strategy);

	if (unlikely(align == SIZE_MAX))
		align = alignof(max_align_t);
	if (unlikely(old_align == SIZE_MAX))
		old_align = alignof(max_align_t);

	if (!layout_requirements_ok(size, align))
		return nullptr;

	struct subregion_node* const node = node_at(find_header((uintptr_t)ptr));
	struct subregion_node* const next =
	  node == allocator->list.tail
		? nullptr
		: node_at((uintptr_t)node, subregion_page_count(node), OFFSET_KIND_PAGE);
	if (unlikely(old_size == SIZE_MAX))
		old_size = subregion_page_count(node) * PAGE_SIZE;

	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum  = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;
	const size_t absolute_page_count_minimum =
	  (absolute_minimum / PAGE_SIZE) + (absolute_minimum % PAGE_SIZE ? 1 : 0);

	// TODO: we might as well split if possible/needed
	if (subregion_page_count(node) >= absolute_page_count_minimum)
	{
		void* new_ptr = (zerOS_byte_t*)node + MEMNODE_ALLOCATED_SIZE + alignment_padding;
		if (new_ptr != ptr)
			memmove(new_ptr, ptr, min(old_size, size));
		return new_ptr;
	}

	zerOS_spin_lock(&allocator->spinlock);

	void* res = nullptr;

	if (
	  next
	  && subregion_free(next)
	  && subregion_page_count(node) + subregion_page_count(next) >= absolute_page_count_minimum)
	{
		res = ptr;

		// NOTE: there shouldn't be multiple adjacent free regions as we should have coalesced them
		const size_t total        = subregion_page_count(node) + subregion_page_count(next);
		const size_t unused_pages = total - absolute_page_count_minimum;

		struct subregion_node* const next_next = subregion_list_next(next, node);
		subregion_list_delete(&allocator->list, next, node, next_next);
		subregion_rbtree_delete(next);

		if (unused_pages != 0)
		{
			subregion_set_page_count(node, absolute_page_count_minimum);
			subregion_split(allocator, node, absolute_page_count_minimum, unused_pages);
		}
		else
			subregion_set_page_count(node, total);

		zerOS_spin_unlock(&allocator->spinlock);
	}
	else
	{
		zerOS_spin_unlock(&allocator->spinlock);

		res = zerOS_region_allocator_alloc(allocator, size, align, strategy);
		if (res)
		{
			memcpy(res, ptr, min(old_size, size));
			zerOS_region_allocator_dealloc(allocator, ptr);
		}
	}

	return res;
}

extern bool zerOS_region_allocator_is_static_region(struct zerOS_region_allocator* allocator)
{
	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	return allocator->is_static_memory;
}

static inline bool rbtree_has_exactly_nelems(struct zerOS_rbtree* const rbtree, const size_t nelems)
{
	size_t                    i    = 0;
	struct zerOS_rbtree_head* head = zerOS_rbtree_get_min(rbtree);
	while (head)
	{
		++i;
		if (i > nelems)
			return false;

		struct subregion_node* node = subregion_node_from_rbtree_head(head);
		if (likely(node->rbtree_info.is_bucket_head))
		{
			node = node->rbtree_info.as_head.list.head;
			while (node)
			{
				++i;
				if (i > nelems)
					return false;

				node = node->rbtree_info.as_member.next;
			}
		}
		head = zerOS_rbtree_get_next(head);
	}

	if (i != nelems)
		return false;

	return true;
}

static inline bool region_is_free_expensive(struct zerOS_region_allocator* allocator)
{
	return !zerOS_rbtree_is_empty(&allocator->rbtree)
		&& rbtree_has_exactly_nelems(&allocator->rbtree, 1)
		&& subregion_page_count(
			 subregion_node_from_rbtree_head(zerOS_rbtree_get_base(&allocator->rbtree)))
			   + 1
			 == allocator->region_page_count;
}

static inline bool region_is_free(struct zerOS_region_allocator* allocator)
{
	return allocator->list.head == allocator->list.tail;
}

extern bool zerOS_region_allocator_reclaim(struct zerOS_region_allocator* allocator)
{
	if (!zerOS_spin_try_lock(&allocator->spinlock))
		return false;

	bool returned = false;
	if (
	  allocator->authorize_reclaim
	  && region_is_free(allocator)
	  && region_is_free_expensive(allocator))
		returned = allocator->reclaim_hook(
		  allocator->region, allocator->region_page_count, allocator->is_static_memory);

	zerOS_spin_unlock(&allocator->spinlock);
	return returned;
}

#if 0
extern struct zerOS_region_allocator*
zerOS_region_allocator_prev(struct zerOS_region_allocator* allocator)
{
	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	return allocator->prev;
}

extern struct zerOS_region_allocator*
zerOS_region_allocator_next(struct zerOS_region_allocator* allocator)
{
	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	return allocator->next;
}

extern void zerOS_region_allocator_set_prev(
  struct zerOS_region_allocator* allocator, struct zerOS_region_allocator* prev)
{
	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	allocator->prev = prev;
}

extern void zerOS_region_allocator_set_next(
  struct zerOS_region_allocator* allocator, struct zerOS_region_allocator* next)
{
	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	allocator->next = next;
}
#endif

extern struct zerOS_region_allocator_additional_space_info
zerOS_region_allocator_additional_space(struct zerOS_region_allocator* allocator)
{
	return (struct zerOS_region_allocator_additional_space_info){
		.addr = allocator->region + sizeof(struct zerOS_region_allocator),
		.size = PAGE_SIZE - sizeof(struct zerOS_region_allocator)
	};
}

extern size_t
zerOS_region_allocator_max_size_for(struct zerOS_region_allocator* allocator, void* ptr)
{
#if 0
	if (unlikely(!zerOS_region_allocator_contains(allocator, ptr)))
		return SIZE_MAX;
#else
	assert(zerOS_region_allocator_contains(allocator, ptr));
#endif

	zerOS_guard(spinlock)(guard, &allocator->spinlock);
	struct subregion_node* node = node_at(find_header((uintptr_t)ptr));
	return subregion_size(node);
}
