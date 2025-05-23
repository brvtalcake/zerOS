// TODO: libdivide is only available for x86-64 targets...
// TODO: random idea I had when writing code here: write a clang plugin to optimize data structure
//       layout for structure marked with a custom attribute

#include <libdivide.h>
#include <region_allocator.h>
#include <stdalign.h>
#include <stdatomic.h>

#undef fast_log2_approx
#define fast_log2_approx(X) \
	((unsigned)(__CHAR_BIT__ * sizeof(typeof((X))) - __builtin_clzg((X), 0)))

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
#define assume(cond) __builtin_assume((bool)(cond))

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

struct subregion_node
{
	// this remains in memory even when allocated, hence it shall never be overwritten by user
	struct [[gnu::packed]] subregion_node_persistent
	{
		uintptr_t xored_prev_next : MAX_VIRTUAL_ADDRESS_LOG2;
		bool      free : 1;
		size_t    page_count : (MAX_VIRTUAL_ADDRESS_LOG2 - PAGE_SIZE_LOG2);
	} persistent;

	// this can be overwritten by user when it has been allocated, hence it is not marked with
	// `[[gnu::packed]]`, since it won't take too much user space on user allocated pages
	struct subregion_node_rbtree_info
	{
		struct subregion_node* parent;
		struct subregion_node* left;
		struct subregion_node* right;
		enum rbtree_color      color;
	} rbt_info;
};

[[gnu::pure]]
static inline size_t subregion_page_count(const struct subregion_node* const restrict node)
{
	return node->persistent.page_count;
}

static inline void subregion_set_page_count(struct subregion_node* node, size_t new_count)
{
	node->persistent.page_count = new_count;
}

[[gnu::pure]]
static inline bool subregion_free(const struct subregion_node* const restrict node)
{
	return node->persistent.free;
}

static inline void subregion_set_free(struct subregion_node* node, bool new_value)
{
	node->persistent.free = new_value;
}

[[clang::overloadable]]
static inline struct subregion_node* node_at(zerOS_byte_t* addr)
{
	return on_page_start((struct subregion_node*)addr);
}

[[clang::overloadable]]
static inline struct subregion_node* node_at(uintptr_t addr)
{
	return node_at((zerOS_byte_t*)addr);
}

[[clang::overloadable]]
static inline struct subregion_node* node_at(void* addr)
{
	return node_at((zerOS_byte_t*)addr);
}

enum offset_kind
{
	OFFSET_KIND_PAGE,
	OFFSET_KIND_RAW
};

[[clang::overloadable]]
static inline struct subregion_node*
node_at(zerOS_byte_t* base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	const size_t multiplier = offkind == OFFSET_KIND_PAGE ? PAGE_SIZE : 1;
	return node_at(base_addr + (offset * multiplier));
}

[[clang::overloadable]]
static inline struct subregion_node*
node_at(uintptr_t base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	return node_at((zerOS_byte_t*)base_addr, offset, offkind);
}

[[clang::overloadable]]
static inline struct subregion_node*
node_at(void* base_addr, ptrdiff_t offset, enum offset_kind offkind)
{
	return node_at((zerOS_byte_t*)base_addr, offset, offkind);
}

struct subregion_list
{
	struct subregion_node* head;
	struct subregion_node* tail;
};

[[gnu::pure]]
static inline struct subregion_node* subregion_list_prev(
  const struct subregion_node* const restrict node,
  const struct subregion_node* const restrict next)
{
	return node_at(node->persistent.xored_prev_next ^ (uintptr_t)next);
}

[[gnu::pure]]
static inline struct subregion_node* subregion_list_next(
  const struct subregion_node* const restrict node,
  const struct subregion_node* const restrict prev)
{
	return node_at(node->persistent.xored_prev_next ^ (uintptr_t)prev);
}

static inline void subregion_list_set_prev_next(
  struct subregion_node* node, struct subregion_node* prev, struct subregion_node* next)
{
	node->persistent.xored_prev_next = (uintptr_t)prev ^ (uintptr_t)next;
}

static inline struct subregion_list subregion_list_new(struct subregion_node* node)
{
	subregion_list_set_prev_next(node, nullptr, nullptr);
	return (struct subregion_list){ .head = node, .tail = node };
}

static inline void subregion_list_insert_before(
  struct subregion_list* list, struct subregion_node* at, struct subregion_node* new_node)
{
	enum
	{
		LIST_PREV = 0,
		LIST_CURR,
		LIST_NEXT
	};

	struct subregion_node* const tmp =
	  at == list->tail ? nullptr
					   : node_at((uintptr_t)at, at->persistent.page_count, OFFSET_KIND_PAGE);
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
  struct subregion_list* list, struct subregion_node* at, struct subregion_node* new_node)
{
	enum
	{
		LIST_PREV = 0,
		LIST_CURR,
		LIST_NEXT
	};

	struct subregion_node* const tmp =
	  at == list->tail ? nullptr
					   : node_at((uintptr_t)at, at->persistent.page_count, OFFSET_KIND_PAGE);
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

/*
 * rbtree implementation inspired from
 * [Wikipedia](https://en.wikipedia.org/wiki/Red%E2%80%93black_tree#Implementation)
 */
struct subregion_rbtree
{
	struct subregion_node* root;
	size_t                 count;
};

[[gnu::pure]]
static inline size_t
subregion_rbtree_max_height(const struct subregion_rbtree* const restrict rbtree)
{
	if (rbtree->count == 0)
		return 1;
	return (2 * fast_log2_approx(rbtree->count + 1)) + 1;
}

[[gnu::pure]]
static inline struct subregion_node*
subregion_rbtree_parent(const struct subregion_node* const restrict node)
{
	return node_at(node->rbt_info.parent);
}

[[gnu::pure]]
static inline struct subregion_node*
subregion_rbtree_left(const struct subregion_node* const restrict node)
{
	return node_at(node->rbt_info.left);
}

[[gnu::pure]]
static inline struct subregion_node*
subregion_rbtree_right(const struct subregion_node* const restrict node)
{
	return node_at(node->rbt_info.right);
}

[[gnu::pure]]
static inline struct subregion_node*
subregion_rbtree_child(const struct subregion_node* const restrict node, enum rbtree_direction dir)
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

[[gnu::pure]]
static inline size_t subregion_rbtree_child_count(const struct subregion_node* const restrict node)
{
	size_t has_left  = (!subregion_rbtree_left(node)) ? 0 : 1;
	size_t has_right = (!subregion_rbtree_right(node)) ? 0 : 1;
	return has_left + has_right;
}

[[gnu::pure]]
static inline enum rbtree_color
subregion_rbtree_color(const struct subregion_node* const restrict node)
{
	return node->rbt_info.color;
}

static inline void
subregion_rbtree_set_parent(struct subregion_node* node, struct subregion_node* new_node)
{
	node->rbt_info.parent = new_node;
}

static inline void
subregion_rbtree_set_left(struct subregion_node* node, struct subregion_node* new_node)
{
	node->rbt_info.left = new_node;
}

static inline void
subregion_rbtree_set_right(struct subregion_node* node, struct subregion_node* new_node)
{
	node->rbt_info.right = new_node;
}

static inline void subregion_rbtree_set_child(
  struct subregion_node* node, struct subregion_node* new_node, enum rbtree_direction dir)
{
	switch (dir)
	{
		case RBTREE_LEFT:
			subregion_rbtree_set_left(node, new_node);
			break;
		case RBTREE_RIGHT:
			subregion_rbtree_set_right(node, new_node);
			break;
		default:
			unreachable();
			break;
	}
}

static inline void subregion_rbtree_set_color(struct subregion_node* node, enum rbtree_color color)
{
	node->rbt_info.color = color;
}

static inline struct subregion_rbtree subregion_rbtree_new(struct subregion_node* node)
{
	subregion_set_free(node, true);
	subregion_rbtree_set_parent(node, nullptr);
	subregion_rbtree_set_left(node, nullptr);
	subregion_rbtree_set_right(node, nullptr);
	subregion_rbtree_set_color(node, RBTREE_BLACK);
	return (struct subregion_rbtree){ .count = 1, .root = node };
}

#undef RBTREE_SELF_DIRECTION
#define RBTREE_SELF_DIRECTION(node) \
	((node) == subregion_rbtree_right(subregion_rbtree_parent((node))) ? RBTREE_RIGHT : RBTREE_LEFT)

#undef RBTREE_IS_BLACK
#undef RBTREE_IS_RED
#define RBTREE_IS_BLACK(node) ((node) == nullptr || subregion_rbtree_color((node)) == RBTREE_BLACK)
#define RBTREE_IS_RED(node)   ((node) != nullptr && subregion_rbtree_color((node)) == RBTREE_RED)

#undef RBTREE_HAS_CHILD
#define RBTREE_HAS_CHILD(node, dir) \
	(subregion_rbtree_child((node), PP_PASTE(RBTREE_, dir)) != nullptr)

static struct subregion_node* subregion_rbtree_rotate_subtree(
  struct subregion_rbtree* tree, struct subregion_node* sub, enum rbtree_direction dir)
{
	struct subregion_node* sub_parent = subregion_rbtree_parent(sub);
	struct subregion_node* new_root =
	  subregion_rbtree_child(sub, 1 - dir); // 1 - dir is the opposite direction
	struct subregion_node* new_child = subregion_rbtree_child(new_root, dir);

	subregion_rbtree_set_child(sub, new_child, 1 - dir);

	if (new_child)
		subregion_rbtree_set_parent(new_child, sub);

	subregion_rbtree_set_child(new_root, sub, dir);

	subregion_rbtree_set_parent(new_root, sub_parent);
	subregion_rbtree_set_parent(sub, new_root);
	if (sub_parent)
		subregion_rbtree_set_child(sub_parent, new_root, sub == subregion_rbtree_right(sub_parent));
	else
		tree->root = new_root;

	return new_root;
}

static inline void
subregion_rbtree_insert_root(struct subregion_rbtree* rbtree, struct subregion_node* node)
{
	*rbtree = subregion_rbtree_new(node);
}

struct insertion_point
{
	struct subregion_node* inspoint;
	enum rbtree_direction  child_dir;
};

// # Prerequisites
// - `rbtree->root` must not be `nullptr`
static inline struct insertion_point subregion_rbtree_non_empty_find_insertion_point(
  const struct subregion_rbtree* const rbtree, const struct subregion_node* const node)
{
	struct insertion_point ret = { .inspoint = rbtree->root };

	while (true)
	{
		ret.child_dir = subregion_page_count(node) <= subregion_page_count(ret.inspoint)
						? RBTREE_LEFT
						: RBTREE_RIGHT;

		struct subregion_node* child = subregion_rbtree_child(ret.inspoint, ret.child_dir);
		if (!child)
			// we got a leaf
			break;

		ret.inspoint = child;
	}

	return ret;
}

// TODO: rewrite the balancing part myself instead of just copy-pasting™ from Wikipedia
static void subregion_rbtree_insert(struct subregion_rbtree* rbtree, struct subregion_node* node)
{
	// TODO: maybe prefetch nodes if it is a bottleneck

	// NOTE: if `rbtree->count != 0`, then `rbtree->root == nullptr` is nearly impossible (normally,
	// at least)
	if (rbtree->count == 0 || unlikely(!rbtree->root))
	{
		subregion_rbtree_insert_root(rbtree, node);
		return;
	}

	struct insertion_point location = subregion_rbtree_non_empty_find_insertion_point(rbtree, node);
	struct subregion_node* parent   = location.inspoint;
	enum rbtree_direction  dir      = location.child_dir;

	subregion_set_free(node, true);
	subregion_rbtree_set_color(node, RBTREE_RED);
	subregion_rbtree_set_parent(node, parent);
	subregion_rbtree_set_left(node, nullptr);
	subregion_rbtree_set_right(node, nullptr);

	subregion_rbtree_set_child(parent, node, dir);

	// rebalance the tree
	do
	{
		// Case #1
		if (RBTREE_IS_BLACK(parent))
			goto deferred;

		struct subregion_node* grandparent = subregion_rbtree_parent(parent);
		if (!grandparent)
		{
			// Case #4
			subregion_rbtree_set_color(parent, RBTREE_BLACK);
			goto deferred;
		}

		dir                          = RBTREE_SELF_DIRECTION(parent);
		struct subregion_node* uncle = subregion_rbtree_child(grandparent, 1 - dir);
		if (RBTREE_IS_BLACK(uncle))
		{
			if (node == subregion_rbtree_child(parent, 1 - dir))
			{
				// Case #5
				subregion_rbtree_rotate_subtree(rbtree, parent, dir);
				node   = parent;
				parent = subregion_rbtree_child(grandparent, dir);
			}

			// Case #6
			subregion_rbtree_rotate_subtree(rbtree, grandparent, 1 - dir);
			subregion_rbtree_set_color(parent, RBTREE_BLACK);
			subregion_rbtree_set_color(grandparent, RBTREE_RED);
			goto deferred;
		}

		// Case #2
		subregion_rbtree_set_color(parent, RBTREE_BLACK);
		subregion_rbtree_set_color(uncle, RBTREE_BLACK);
		subregion_rbtree_set_color(grandparent, RBTREE_RED);
		node = grandparent;

	} while (parent = subregion_rbtree_parent(node));

deferred:
	rbtree->count += 1;
}

static inline struct subregion_node*
subregion_rbtree_get_leftmost_in_subtree(struct subregion_node* node)
{
	struct subregion_node* left;
	while (left = subregion_rbtree_left(node))
		node = left;
	return node;
}

static void subregion_rbtree_replace(struct subregion_node* node, struct subregion_node* new_node)
{
	struct subregion_node* left;
	struct subregion_node* right;
	struct subregion_node* parent;
	enum rbtree_direction  pdir;
	enum rbtree_color      pcol;

	*new_node = *node;

	left   = subregion_rbtree_child(node, RBTREE_LEFT);
	right  = subregion_rbtree_child(node, RBTREE_RIGHT);
	parent = subregion_rbtree_parent(node);
	pdir   = RBTREE_SELF_DIRECTION(node);

	if (left)
		subregion_rbtree_set_parent(left, new_node);
	if (right)
		subregion_rbtree_set_parent(right, new_node);
	subregion_rbtree_set_child(parent, new_node, pdir);
}

static inline void subregion_rbtree_connect_maybe_null(
  struct subregion_node* parent,
  enum rbtree_direction  dir,
  struct subregion_node* child,
  enum rbtree_color      color)
{
	subregion_rbtree_set_child(parent, child, dir);
	if (child)
	{
		subregion_rbtree_set_parent(child, parent);
		subregion_rbtree_set_color(child, color);
	}
}

static inline void subregion_rbtree_connect(
  struct subregion_node* parent,
  enum rbtree_direction  dir,
  struct subregion_node* child,
  enum rbtree_color      color)
{
#if 0
	subregion_rbtree_set_child(parent, child, dir);
	subregion_rbtree_set_parent(child, parent);
	subregion_rbtree_set_color(child, color);
#else
	assume(child);
	subregion_rbtree_connect_maybe_null(parent, dir, child, color);
#endif
}

// see the comment for `subregion_rbtree_remove_noninternal`
static void
subregion_rbtree_rebalance_after_unlink(struct subregion_node* pnt, enum rbtree_direction pdir)
{
	struct subregion_node* gpnt;
	struct subregion_node* sibling;
	struct subregion_node* sleft;
	struct subregion_node* sright;
	struct subregion_node* sleftleft;
	struct subregion_node* sleftright;
	enum rbtree_direction  left;
	enum rbtree_direction  right;
	enum rbtree_direction  gdir;

	if (!subregion_rbtree_parent(pnt))
		return;

	left    = pdir; // define "left" as the direction from parent to deleted node
	right   = pdir != RBTREE_LEFT ? RBTREE_LEFT : RBTREE_RIGHT;
	gpnt    = subregion_rbtree_parent(pnt);
	gdir    = RBTREE_SELF_DIRECTION(pnt);
	sibling = subregion_rbtree_child(pnt, right);
	sleft   = subregion_rbtree_child(sibling, left);
	sright  = subregion_rbtree_child(sibling, right);

	if (RBTREE_IS_RED(sibling))
	{
		/* sibling is red */
		subregion_rbtree_connect(pnt, right, sleft, RBTREE_BLACK);
		subregion_rbtree_connect(sibling, left, pnt, RBTREE_RED);
		subregion_rbtree_connect(gpnt, gdir, sibling, RBTREE_BLACK);
		subregion_rbtree_rebalance_after_unlink(pnt, pdir);
	}
	else if (RBTREE_IS_RED(sright))
	{
		/* outer child of sibling is red */
		enum rbtree_color pntcol = subregion_rbtree_color(pnt);
		subregion_rbtree_connect_maybe_null(pnt, right, sleft, subregion_rbtree_color(sleft));
		subregion_rbtree_connect(sibling, left, pnt, RBTREE_BLACK);
		subregion_rbtree_connect(gpnt, gdir, sibling, pntcol);
		rb3_set_black(sibling, right);
	}
	else if (rb3_is_red(sibling, left))
	{
		/* inner child of sibling is red */
		sleftleft  = rb3_get_child(sleft, left);
		sleftright = rb3_get_child(sleft, right);
		rb3_connect_null(pnt, right, sleftleft, RBTREE_BLACK);
		rb3_connect_null(sibling, left, sleftright, RBTREE_BLACK);
		rb3_connect(sleft, left, pnt, RBTREE_BLACK);
		rb3_connect(sleft, right, sibling, RBTREE_BLACK);
		rb3_connect(gpnt, gdir, sleft, rb3_get_color_bit(gpnt, gdir));
		if (augment)
		{
			augment(sibling);
			rb3_update_augment(pnt, augment);
		}
	}
	else if (rb3_is_red(gpnt, gdir))
	{
		/* parent is red */
		rb3_set_red(pnt, right);
		rb3_set_black(gpnt, gdir);
		if (augment)
			rb3_update_augment(pnt, augment);
	}
	else
	{
		/* all relevant nodes are black */
		rb3_set_red(pnt, right);
		if (augment)
			augment(pnt);
		subregion_rbtree_rebalance_after_unlink(gpnt, gdir, augment);
	}
}

// copied from
// https://github.com/jstimpfle/rb3ptr/blob/faf0b609b35a2183df28c62d28afb541c3c130fb/rb3ptr.c#L313
static void
subregion_rbtree_remove_noninternal(struct subregion_rbtree* rbtree, struct subregion_node* node)
{
	struct subregion_node* pnt;
	struct subregion_node* cld;
	enum rbtree_direction  pdir;
	enum rbtree_direction  dir;

	dir  = subregion_rbtree_child(node, RBTREE_RIGHT) ? RBTREE_RIGHT : RBTREE_LEFT;
	pnt  = subregion_rbtree_parent(node);
	cld  = subregion_rbtree_child(node, dir);
	pdir = RBTREE_SELF_DIRECTION(node);

	struct subregion_node* to_test1       = subregion_rbtree_child(pnt, pdir);
	struct subregion_node* to_test2       = subregion_rbtree_child(node, dir);
	bool                   must_rebalance = !RBTREE_IS_RED(to_test1) && !RBTREE_IS_RED(to_test2);

	/* since we added the possibility for augmentation,
	we need to remove `head` *before* the rebalancing that we do below.
	(Otherwise the augmentation function would still see the to-be-deleted child). */
	subregion_rbtree_connect_maybe_null(pnt, pdir, cld, RBTREE_BLACK);
	// subregion_rbtree_set_child(pnt, cld, pdir);
	// if (child)
	//{
	//	subregion_rbtree_set_parent(cld, pnt);
	//	subregion_rbtree_set_color(cld, RBTREE_BLACK);
	// }

	if (must_rebalance)
		/* To be deleted node is black (and child cannot be repainted)
		 * => height decreased */
		subregion_rbtree_rebalance_after_unlink(pnt, pdir);
}

static void
subregion_rbtree_remove_internal(struct subregion_rbtree* rbtree, struct subregion_node* node)
{
	struct subregion_node* replacement =
	  subregion_rbtree_get_leftmost_in_subtree(subregion_rbtree_right(node));
	subregion_rbtree_remove_noninternal(rbtree, replacement);
	subregion_rbtree_replace(node, replacement);
}

// TODO: (same comment as for `subregion_rbtree_insert`) rewrite the balancing part myself instead
// of just copy-pasting™ from Wikipedia
// TODO: this function is straight-up wrong
static void subregion_rbtree_remove(struct subregion_rbtree* rbtree, struct subregion_node* node)
{
	subregion_set_free(node, false);

	const size_t child_count = subregion_rbtree_child_count(node);
	switch (child_count)
	{
		case 0:
			[[__fallthrough__]];
		case 1:
			subregion_rbtree_remove_noninternal(rbtree, node);
			break;
		case 2: {
			subregion_rbtree_remove_internal(rbtree, node);
		}
		default:
			unreachable();
			break;
	}

deferred:
	rbtree->count -= 1;
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
static constexpr unsigned MEMNODE_ALLOCATED_SIZE = sizeof(struct subregion_node_persistent);

static void subregion_split(
  struct zerOS_region_allocator* manager,
  struct subregion_node*         node,
  size_t                         new_page_offset,
  size_t                         new_page_count)
{
	struct subregion_node* new_node = node_at((uintptr_t)node, new_page_offset, OFFSET_KIND_PAGE);
	subregion_set_page_count(new_node, new_page_count);

	subregion_list_insert_after(&manager->list, node, new_node);
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
	return libdivide_u32_branchfree_do(numer, &wrapper->divider);
#elif SIZE_WIDTH == 64
	return libdivide_u64_branchfree_do(numer, &wrapper->divider);
#endif
}

static void fast_divmod(union fast_divmod* inout)
{
	// clang-format off
	// NOTE: `fast_dividers[0]` corresponds to the divider for
	//		 `denom == 1`
	static struct libdivide_wrapper fast_dividers[(MAX_ALIGN - 1)] = {
		[0 ... (MAX_ALIGN - 1) - 1] = { .initialized = false }
	};
	// clang-format on

	const size_t numer = inout->in.numer;
	const size_t denom = inout->in.denom;

	struct libdivide_wrapper* divider = fast_dividers + denom - 1;

	inout->out.div = libdivide_wrapper_calc(divider, numer, denom);
	inout->out.mod = numer - (inout->out.div * denom);
}

// TODO: align_up and align_down macros or functions

static inline uintptr_t align_up(uintptr_t ptr, size_t align)
{
	static_assert(sizeof(uintptr_t) == sizeof(size_t));
	static_assert(alignof(uintptr_t) == alignof(size_t));
	union fast_divmod divmod = {
		.in = { .numer = ptr, .denom = align }
	};
	fast_divmod(&divmod);
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

	subregion_set_page_count(first_node, (region_size / PAGE_SIZE) - 1);

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
static inline struct subregion_node* subregion_rbtree_inorder_traverse_from_until_impl(
  const size_t from,
  const bool   lower_bound_included,
  bool         (*const until_predicate)(const struct subregion_node*, void*),
  struct subregion_node* restrict node,
  void* user_data)
{
	struct subregion_node*       next_visited;
	struct subregion_node* const left  = subregion_rbtree_left(node);
	struct subregion_node* const right = subregion_rbtree_right(node);

	assume_unaliasing_pointers(left, right);
	assume_unaliasing_pointers(node, right);
	assume_unaliasing_pointers(left, node);
	prefetch_rw(left, 2);
	prefetch_rw(right, 3);

	const bool current_is_big_enough =
	  from < subregion_page_count(node) * PAGE_SIZE
	  || (lower_bound_included && unlikely(from == subregion_page_count(node) * PAGE_SIZE));

	// NOTE: if the current node is already not enough to hold the required size, there is no point
	// going through the left subtree. On the other hand, it might be okay to through the right one
	// unconditionally, as we could find an even better (greater) match

	if (left && current_is_big_enough)
	{
		if ((from < subregion_page_count(left) * PAGE_SIZE
			 || (lower_bound_included && unlikely(from == subregion_page_count(left) * PAGE_SIZE))))
		{
			next_visited = subregion_rbtree_inorder_traverse_from_until_impl(
			  from, lower_bound_included, until_predicate, left, user_data);
			if (next_visited)
				return next_visited;
		}
	}

	if (current_is_big_enough)
	{
		if (until_predicate(node, user_data))
			return node;
	}

	if (right)
	{
		next_visited = subregion_rbtree_inorder_traverse_from_until_impl(
		  from, lower_bound_included, until_predicate, right, user_data);
		if (next_visited)
			return next_visited;
	}

	return nullptr;
}

static struct subregion_node* subregion_rbtree_inorder_traverse_from_until(
  struct zerOS_region_allocator* allocator,
  const size_t                   from,
  const bool                     lower_bound_included,
  bool                           (*const until_predicate)(const struct subregion_node*, void*),
  void*                          user_data)
{
	prefetch_rw(allocator->rbtree.root, 2);

	struct subregion_node* start = allocator->rbtree.root;

	return subregion_rbtree_inorder_traverse_from_until_impl(
	  from, lower_bound_included, until_predicate, start, user_data);
}

static inline bool suitable_region_predicate(const struct subregion_node* node, void*)
{
	// should always be true if the rbtree contains it
	return likely(subregion_free(node));
}

static inline size_t padding_for(size_t align)
{
	const uintptr_t aligned = align_up(MEMNODE_ALLOCATED_SIZE, align);
	return aligned - MEMNODE_ALLOCATED_SIZE;
}

static void*
region_alloc_best_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	if (!allocator->rbtree.count || !allocator->rbtree.root)
		return nullptr;

	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum  = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;
	const size_t absolute_page_count_minimum =
	  (absolute_minimum / PAGE_SIZE) + (absolute_minimum % PAGE_SIZE ? 1 : 0);

	struct subregion_node* node = subregion_rbtree_inorder_traverse_from_until(
	  allocator, absolute_minimum, true, &suitable_region_predicate, nullptr);
	if (node)
	{
		// we found a chunk
		const size_t base_chunk_page_count = subregion_page_count(node);
		const size_t unused_pages = absolute_page_count_minimum - subregion_page_count(node);

		// this function already sets `node->persistent.free = false`
		subregion_rbtree_remove(&allocator->rbtree, node);

		if (unused_pages > 0)
		{
			subregion_set_page_count(node, absolute_page_count_minimum);
			subregion_split(allocator, node, absolute_page_count_minimum, unused_pages);
		}

		return (zerOS_byte_t*)node + MEMNODE_ALLOCATED_SIZE;
	}

	// failed to get a suitable chunk
	return nullptr;
}

static void*
region_alloc_first_fit(struct zerOS_region_allocator* allocator, size_t size, size_t align)
{
	if (!allocator->rbtree.count || !allocator->rbtree.root)
		return nullptr;

	const size_t alignment_padding = padding_for(align);
	const size_t absolute_minimum  = MEMNODE_ALLOCATED_SIZE + alignment_padding + size;
	const size_t absolute_page_count_minimum =
	  (absolute_minimum / PAGE_SIZE) + (absolute_minimum % PAGE_SIZE ? 1 : 0);

	// get rightmost leaf
	struct subregion_node* node = allocator->rbtree.root;
	while (node && subregion_page_count(node) < absolute_page_count_minimum)
		node = subregion_rbtree_right(node);

	if (node)
	{
		// we found a chunk
		const size_t base_chunk_page_count = subregion_page_count(node);
		const size_t unused_pages = absolute_page_count_minimum - subregion_page_count(node);

		// this function already sets `node->persistent.free = false`
		subregion_rbtree_remove(&allocator->rbtree, node);

		if (unused_pages > 0)
		{
			subregion_set_page_count(node, absolute_page_count_minimum);
			subregion_split(allocator, node, absolute_page_count_minimum, unused_pages);
		}

		return (zerOS_byte_t*)node + MEMNODE_ALLOCATED_SIZE;
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
	if (!layout_requirements_ok(size, align))
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
