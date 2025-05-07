#include <libdivide.h>
#include <region_allocator.h>
#include <stdalign.h>
#include <stdatomic.h>

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

#undef fast_log2_approx
#define fast_log2_approx(X) \
	((unsigned)(__CHAR_BIT__ * sizeof(typeof((X))) - __builtin_clzg((X), 0)))

#undef L1_CACHE_LINE_SIZE
#define L1_CACHE_LINE_SIZE 64

#undef PAGE_SIZE
#define PAGE_SIZE 4096

#undef MAX_ALIGN
#define MAX_ALIGN PAGE_SIZE

#undef MAX_ALIGN_LOG2
#define MAX_ALIGN_LOG2 PAGE_SIZE_LOG2

#undef MAX_ADDRESS_LOG2
#define MAX_ADDRESS_LOG2 57

#undef PAGE_SIZE_LOG2
#define PAGE_SIZE_LOG2 12

#undef likely
#undef unlikely
#undef expect
#define likely(...)         __builtin_expect(!!(__VA_ARGS__), true)
#define unlikely(...)       __builtin_expect(!!(__VA_ARGS__), false)
#define expect(expr, value) __builtin_expect((expr), (value))

#undef prefetch
#undef prefetch_ro
#undef prefetch_rw
#define prefetch(...)          __builtin_prefetch(__VA_ARGS__)
#define prefetch_ro(addr, ...) prefetch((addr), 0, __VA_ARGS__)
#define prefetch_rw(addr, ...) prefetch((addr), 1, __VA_ARGS__)

#undef prefetch_range
#undef prefetch_range_ro
#undef prefetch_range_rw
#define prefetch_range(addr, size, ...)                                                          \
	({                                                                                           \
		char* UNIQUE(addr_in_prefetch_range) = (char*)(addr);                                    \
		for (size_t UNIQUE(i_in_prefetch_range)  = 0; UNIQUE(i_in_prefetch_range) < (size);      \
			 UNIQUE(i_in_prefetch_range)        += L1_CACHE_LINE_SIZE)                           \
		{                                                                                        \
			prefetch(UNIQUE(addr_in_prefetch_range) + UNIQUE(i_in_prefetch_range), __VA_ARGS__); \
		}                                                                                        \
	})
#define prefetch_range_ro(addr, size, ...) prefetch_range((addr), (size), 0, __VA_ARGS__)
#define prefetch_range_rw(addr, size, ...) prefetch_range((addr), (size), 1, __VA_ARGS__)

#undef assume_aligned
#define assume_aligned(ptr, align) __builtin_assume_aligned((ptr), (align))

#undef assume
#define assume(cond) __builtin_assume(!!(cond))

#undef assume_unaliasing_pointers
#define assume_unaliasing_pointers(ptr1, ptr2) __builtin_assume_separate_storage((ptr1), (ptr2))

#undef on_page_start
#define on_page_start(ptr) assume_aligned((ptr), (PAGE_SIZE))

#ifndef alloca
	#define alloca(size) __builtin_alloca(size)
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
		__auto_type UNIQUE(a_in_min) = (a);                                        \
		__auto_type UNIQUE(b_in_min) = (b);                                        \
		UNIQUE(a_in_min) < UNIQUE(b_in_min) ? UNIQUE(a_in_min) : UNIQUE(b_in_min); \
	})
#define max(a, b)                                                                  \
	({                                                                             \
		__auto_type UNIQUE(a_in_max) = (a);                                        \
		__auto_type UNIQUE(b_in_max) = (b);                                        \
		UNIQUE(a_in_max) > UNIQUE(b_in_max) ? UNIQUE(a_in_max) : UNIQUE(b_in_max); \
	})

#undef distance
#define distance(a, b)                                         \
	({                                                         \
		__auto_type UNIQUE(a_in_distance) = (a);               \
		__auto_type UNIQUE(b_in_distance) = (b);               \
		max(UNIQUE(a_in_distance), UNIQUE(b_in_distance))      \
		  - min(UNIQUE(a_in_distance), UNIQUE(b_in_distance)); \
	})

enum rbtree_color
{
	RBTREE_BLACK = 0u,
	RBTREE_RED   = 1u,
};

enum rbtree_direction
{
	RBTREE_LEFT  = 0u,
	RBTREE_RIGHT = 1u,
};

struct [[gnu::packed]] subregion_node
{
	uintptr_t prev : MAX_ADDRESS_LOG2;
	uintptr_t next : MAX_ADDRESS_LOG2;
	bool      free : 1;
	size_t    page_count : (MAX_ADDRESS_LOG2 - PAGE_SIZE_LOG2);

	uintptr_t         left : MAX_ADDRESS_LOG2;
	uintptr_t         right : MAX_ADDRESS_LOG2;
	enum rbtree_color color : 1;
};

struct subregion_list
{
	struct subregion_node* head;
	struct subregion_node* tail;
};

// TODO: maybe accessor functions could be `__attribute__((const))` or `__attribute__((pure))` ?

static inline struct subregion_node* subregion_list_prev(struct subregion_node* node)
{
	return on_page_start((struct subregion_node*)node->prev);
}

static inline struct subregion_node* subregion_list_next(struct subregion_node* node)
{
	return on_page_start((struct subregion_node*)node->next);
}

static inline void subregion_list_set_prev(struct subregion_node* node, struct subregion_node* new)
{
	node->prev = (uintptr_t)new;
}

static inline void subregion_list_set_next(struct subregion_node* node, struct subregion_node* new)
{
	node->next = (uintptr_t)new;
}

static inline struct subregion_list subregion_list_new(struct subregion_node* node)
{
	subregion_list_set_prev(node, nullptr);
	subregion_list_set_next(node, nullptr);
	return (struct subregion_list){ .head = node, .tail = node };
}

static inline void subregion_list_insert_before(
  struct subregion_list* list, struct subregion_node* at, struct subregion_node* node)
{
	struct subregion_node* prev = subregion_list_prev(at);
	if (!prev)
	{
		// `at` is the list `head`
		list->head = node;

		subregion_list_set_prev(node, nullptr);
		subregion_list_set_next(node, at);

		subregion_list_set_prev(at, node);
	}
	else
	{
		// general case
		subregion_list_set_prev(node, prev);
		subregion_list_set_next(node, at);

		subregion_list_set_prev(at, node);
		subregion_list_set_next(prev, node);
	}
}

static inline void subregion_list_insert_after(
  struct subregion_list* list, struct subregion_node* at, struct subregion_node* node)
{
	struct subregion_node* next = subregion_list_prev(at);
	if (!next)
	{
		// `at` is the list `tail`
		list->tail = node;

		subregion_list_set_prev(node, at);
		subregion_list_set_next(node, nullptr);

		subregion_list_set_next(at, node);
	}
	else
	{
		// general case
		subregion_list_set_prev(node, at);
		subregion_list_set_next(node, next);

		subregion_list_set_next(at, node);
		subregion_list_set_prev(next, node);
	}
}

/*
 * rbtree implementation inspired from
 * [Wikipedia](https://en.wikipedia.org/wiki/Red%E2%80%93black_tree#Implementation)
 */
struct subregion_rbtree
{
	struct subregion_node* root;
	size_t                 count;
};

static inline size_t subregion_rbtree_max_height(struct subregion_rbtree* rbtree)
{
	return (2 * fast_log2_approx(rbtree->count + 1)) + 1;
}

static inline struct subregion_node* subregion_rbtree_left(struct subregion_node* node)
{
	return on_page_start((struct subregion_node*)node->left);
}

static inline struct subregion_node* subregion_rbtree_right(struct subregion_node* node)
{
	return on_page_start((struct subregion_node*)node->right);
}

static inline struct subregion_node*
subregion_rbtree_child(struct subregion_node* node, enum rbtree_direction dir)
{
	switch (dir)
	{
		case RBTREE_LEFT:
			return subregion_rbtree_left(node);
		case RBTREE_RIGHT:
			return subregion_rbtree_right(node);
		default:
			unreachable();
	}
}

static inline enum rbtree_color subregion_rbtree_color(struct subregion_node* node)
{
	return node->color;
}

static inline void
subregion_rbtree_set_left(struct subregion_node* node, struct subregion_node* new)
{
	node->left = (uintptr_t)new;
}

static inline void
subregion_rbtree_set_right(struct subregion_node* node, struct subregion_node* new)
{
	node->right = (uintptr_t)new;
}

static inline void subregion_rbtree_set_child(
  struct subregion_node* node, struct subregion_node* new, enum rbtree_direction dir)
{
	switch (dir)
	{
		case RBTREE_LEFT:
			subregion_rbtree_set_left(node, new);
			break;
		case RBTREE_RIGHT:
			subregion_rbtree_set_right(node, new);
			break;
		default:
			unreachable();
			break;
	}
}

static inline void subregion_rbtree_set_color(struct subregion_node* node, enum rbtree_color color)
{
	node->color = color;
}

static inline struct subregion_rbtree subregion_rbtree_new(struct subregion_node* node)
{
	subregion_rbtree_set_left(node, nullptr);
	subregion_rbtree_set_right(node, nullptr);
	subregion_rbtree_set_color(node, RBTREE_BLACK);
	return (struct subregion_rbtree){ .count = 1, .root = node };
}

static struct subregion_node* rotate_subtree(
  struct subregion_rbtree* tree,
  struct subregion_node*   sub,
  struct subregion_node*   sub_parent,
  enum rbtree_direction    dir)
{
	struct subregion_node* new_root =
	  subregion_rbtree_child(sub, 1 - dir); // 1 - dir is the opposite direction
	struct subregion_node* new_child = subregion_rbtree_child(new_root, dir);

	subregion_rbtree_set_child(sub, new_child, 1 - dir);

	// if (new_child)
	//	new_child->parent = sub;

	subregion_rbtree_set_child(new_root, sub, dir);

	// new_root->parent = sub_parent;
	// sub->parent      = new_root;
	if (sub_parent)
		subregion_rbtree_set_child(sub_parent, new_root, sub == subregion_rbtree_right(sub_parent));
	else
		tree->root = new_root;

	return new_root;
}

struct zerOS_region_allocator
{
	struct subregion_rbtree rbtree;
	struct subregion_list   list;

	zerOS_byte_t* region;
	size_t        region_page_count;

	enum zerOS_allocation_strategy preferred_strategy;
	bool                           is_static_memory;
	bool                           authorize_reclaim;
	zerOS_region_reclaim_hook_t    reclaim_hook;
};

/**
 * @brief The size of a `struct subregion_node` when not allocated
 */
static constexpr unsigned MEMNODE_SIZE = sizeof(struct subregion_node);
/**
 * @brief The size of a `struct subregion_node` when allocated
 */
static constexpr unsigned MEMNODE_ALLOCATED_SIZE =
  (2 * MAX_ADDRESS_LOG2 + 1 + MAX_ADDRESS_LOG2 - PAGE_SIZE_LOG2) / __CHAR_BIT__
  + ((2 * MAX_ADDRESS_LOG2 + 1 + MAX_ADDRESS_LOG2 - PAGE_SIZE_LOG2) % __CHAR_BIT__ ? 1 : 0);

static inline struct subregion_node* subregion_free(struct subregion_node* node)
{
	return on_page_start((struct subregion_node*)node->free);
}

static bool subregion_split(
  struct zerOS_region_allocator* manager, struct subregion_node* node, size_t page_count)
{
	(void)manager;
	(void)node;
	(void)page_count;
	// TODO
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

struct libdivide_wrapper
{
#if SIZE_WIDTH == 32
	struct libdivide_u32_branchfree_t fast_dividers[MAX_ALIGN];
#elif SIZE_WIDTH == 64
	struct libdivide_u64_branchfree_t divider;
#else
	#error "unsupported SIZE_WIDTH"
#endif
	bool initialized;
};

static inline void libdivide_wrapper_init(struct libdivide_wrapper* wrapper, size_t denom)
{
	if (wrapper->initialized)
		return;

#if SIZE_WIDTH == 32
	wrapper->divider = libdivide_u32_branchfree_gen(denom);
#elif SIZE_WIDTH == 64
	wrapper->divider = libdivide_u64_branchfree_gen(denom);
#endif
	wrapper->initialized = true;
}

static inline size_t
libdivide_wrapper_calc(struct libdivide_wrapper* wrapper, size_t numer, size_t denom)
{
	if (!wrapper->initialized)
		libdivide_wrapper_init(wrapper, denom);

#if SIZE_WIDTH == 32
	return libdivide_u32_branchfree_do(numer, wrapper->divider);
#elif SIZE_WIDTH == 64
	return libdivide_u64_branchfree_do(numer, wrapper->divider);
#endif
}

static void fast_divmod(union fast_divmod* inout)
{
	// clang-format off
	static struct libdivide_wrapper fast_dividers[(MAX_ALIGN - 1)] = {
		[0 ... (MAX_ALIGN - 1) - 1] = { .initialized = false }
	};
	// clang-format on

	const size_t numer = inout->in.numer;
	const size_t denom = inout->in.denom;

	struct libdivide_wrapper* divider = fast_dividers + denom - 1;

	inout->out.div = libdivide_wrapper_calc(divider, numer, denom);
	inout->out.mod = numer - inout->out.div;
}

// TODO: align_up and align_down macros or functions

static inline uintptr_t align_up(uintptr_t ptr, size_t align)
{
	static_assert(sizeof(uintptr_t) == sizeof(size_t));
	static_assert(alignof(uintptr_t) == alignof(size_t));
	union fast_divmod divmod = {
		.in = { .numer = ptr, .denom = align }
	};
	fast_divmod(divmod);
	return ptr + divmod.out.mod;
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

	struct subregion_node* first_node = on_page_start(region + PAGE_SIZE);

	first_node->page_count = (region_size / PAGE_SIZE) - 1;
	first_node->free       = true;

	struct zerOS_region_allocator* ret = on_page_start(region);

	ret->list   = subregion_list_new(first_node);
	ret->rbtree = subregion_rbtree_new(first_node);

	ret->region            = region;
	ret->region_page_count = region_size / PAGE_SIZE;

	ret->is_static_memory = static_mem;
	ret->preferred_strategy =
	  preferred != zerOS_ALLOC_STRAT_DEFAULT ? preferred : zerOS_ALLOC_STRAT_BEST_FIT;

	if (authorize_reclaim && hook != nullptr)
	{
		ret->authorize_reclaim = true;
		ret->reclaim_hook      = hook;
	}
	else
	{
		ret->authorize_reclaim = false;
		ret->reclaim_hook      = nullptr;
	}

	return ret;
}

/**
 * @brief Classic recursive in-order traversal, but only visiting in-bounds nodes/subtrees
 *
 * @todo param tnode The subtree node to start recursing from.
 * @param from The lower bound.
 * @param lower_bound_included Is the lower bound included ? Else it is considered excluded.
 * @param until_predicate Stop traversing as soon as this predicate holds for a specific node.
 * @param node The node at which the recursion stopped, if the function returns true. Else, an
 * unuspecified value.
 * @param parent The node's parent (if any, else `nullptr`) at which the recursion stopped, if the
 * function returns true. Else, an unuspecified value.
 * @param grand_parent The node's grand-parent (if any, else `nullptr`) at which the recursion
 * stopped, if the function returns true. Else, an unuspecified value.
 * @retval `true`: If the function succeded.
 * @retval `false`: If the function failed.
 */
static inline bool subregion_rbtree_inorder_traverse_from_until_impl(
  const size_t from,
  const bool   lower_bound_included,
  bool         (*const until_predicate)(const struct subregion_node*, size_t, size_t),
  struct subregion_node** restrict node,
  struct subregion_node** restrict parent,
  struct subregion_node** restrict grand_parent,
  const size_t size,
  const size_t align)
{
	struct subregion_node* const current = *node;
	struct subregion_node* const left    = subregion_rbtree_left(current);
	struct subregion_node* const right   = subregion_rbtree_right(current);

	assume_unaliasing_pointers(left, right);
	assume_unaliasing_pointers(current, right);
	assume_unaliasing_pointers(left, current);
	prefetch_rw(left, 2);
	prefetch_rw(right, 3);

	const bool current_is_big_enough =
	  from < current->page_count * PAGE_SIZE
	  || (lower_bound_included && unlikely(from == current->page_count * PAGE_SIZE));

	// NOTE: if the current node is already not enough to hold the required size, there is no point
	// going through the left subtree. On the other hand, it might be okay to through the right one
	// unconditionally, as we could find an even better (greater) match

	if (left && current_is_big_enough)
	{
		if ((from < left->page_count * PAGE_SIZE
			 || (lower_bound_included && unlikely(from == left->page_count * PAGE_SIZE))))
		{
			struct subregion_node* new              = left;
			struct subregion_node* new_parent       = *node;
			struct subregion_node* new_grand_parent = *parent;
			if (subregion_rbtree_inorder_traverse_from_until_impl(
				  from, lower_bound_included, until_predicate, &new, &new_parent, &new_grand_parent,
				  size, align))
			{
				*node         = new;
				*parent       = new_parent;
				*grand_parent = new_grand_parent;
				return true;
			}
		}
	}

	if (current_is_big_enough)
	{
		if (until_predicate(*node, size, align))
			return true;
	}

	if (right)
	{
		struct subregion_node* new              = right;
		struct subregion_node* new_parent       = *node;
		struct subregion_node* new_grand_parent = *parent;
		if (subregion_rbtree_inorder_traverse_from_until_impl(
			  from, lower_bound_included, until_predicate, &new, &new_parent, &new_grand_parent,
			  size, align))
		{
			*node         = new;
			*parent       = new_parent;
			*grand_parent = new_grand_parent;
			return true;
		}
	}
}

static bool subregion_rbtree_inorder_traverse_from_until(
  struct zerOS_region_allocator* allocator,
  const size_t                   from,
  const bool                     lower_bound_included,
  bool (*const until_predicate)(const struct subregion_node*, size_t, size_t),
  struct subregion_node** restrict node,
  struct subregion_node** restrict parent,
  struct subregion_node** restrict grand_parent,
  const size_t size,
  const size_t align)
{
	prefetch_rw(allocator->rbtree.root, 2);

	*node         = allocator->rbtree.root;
	*parent       = nullptr;
	*grand_parent = nullptr;

	return subregion_rbtree_inorder_traverse_from_until_impl(
	  from, lower_bound_included, until_predicate, node, parent, grand_parent, size, align);
}

static inline bool
suitable_region_predicate(const struct subregion_node* node, size_t size, size_t align)
{
	if (likely(node->free))
	{
		return true;
	}
	else
		return false;
}

static inline size_t padding_for(size_t align)
{
	const uintptr_t aligned = align_up(MEMNODE_ALLOCATED_SIZE, align);
	return aligned - MEMNODE_ALLOCATED_SIZE;
}

static void*
region_alloc_best_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;

	struct subregion_node* node;
	struct subregion_node* parent;
	struct subregion_node* grand_parent;

	if (subregion_rbtree_inorder_traverse_from_until(
		  allocator, absolute_minimum, true, &suitable_region_predicate, &node, &parent,
		  &grand_parent, size, align))
	{
		// all good
		// process...
	}

	// failed to get a suitable chunk
	return nullptr;
}

static void*
region_alloc_first_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	return nullptr;
}

extern void* zerOS_region_allocator_alloc(
  struct zerOS_region_allocator* allocator,
  size_t                         size,
  size_t                         align,
  enum zerOS_allocation_strategy strategy)
{
	if (unlikely(align > MAX_ALIGN) || unlikely(align == 0))
		return nullptr;

	prefetch_rw(allocator, 2);

	enum zerOS_allocation_strategy strat =
	  strategy != zerOS_ALLOC_STRAT_DEFAULT ? strategy : allocator->preferred_strategy;

	switch (strat)
	{
		case zerOS_ALLOC_STRAT_BEST_FIT:
			return region_alloc_best_fit(allocator, size, align);
		case zerOS_ALLOC_STRAT_FIRST_FIT:
			return region_alloc_first_fit(allocator, size, align);
		default:
			unreachable();
			break;
	}

	return nullptr;
}
