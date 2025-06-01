/* zerOS_rbtreeptr  -- Intrusively linked 3-pointer Red-black tree implementation */

/* Copyright (C) 2019, Jens Stimpfle */

/*
Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

#ifndef zerOS_RBTREE_H_INCLUDED_
#define zerOS_RBTREE_H_INCLUDED_ 1

#include <zerOS/common.h>

#undef zerOS_RBTREE_NULL
#define zerOS_RBTREE_NULL nullptr

#ifdef __cplusplus // not yet tested
extern "C"
{
#endif

/**
 * Directions for navigation in the tree.
 */
enum zerOS_rbtree_direction
{
	zerOS_RBTREE_LEFT  = 0u,
	zerOS_RBTREE_RIGHT = 1u,
};

#if 0
enum zerOS_rbtree_color
{
	zerOS_RBTREE_BLACK = 0u,
	zerOS_RBTREE_RED   = 1u,
};
#endif

/**
 * This type is used to efficiently store a pointer (at least 4-byte aligned)
 * and some more information in the unused low bits.
 */
typedef uintptr_t zerOS_rbtree_ptr;

/**
 * Node type for 3-pointer Red-black trees.
 * Contains left, right, and parent pointers.
 * The left and right pointers have additional color bits.
 * The parent pointer contains a direction bit indicating the direction
 * to this child.
 */
struct zerOS_rbtree_head
{
	zerOS_rbtree_ptr child[2];
	zerOS_rbtree_ptr parent;
};

/**
 * Tree type. It's just a fake base head that is wrapped for type safety and
 * future extensibility.
 */
struct zerOS_rbtree
{
	struct zerOS_rbtree_head base;
};

/**
 * User-provided comparison function. It is used during tree searches.
 * At each visited node, the function is called with that node as first
 * argument and some additional user-provided data.
 *
 * It should returns a value less than, equal to, or greater than, 0,
 * depending on whether the node compares less than, equal to, or greater
 * than, the user-provided data.
 */
typedef int (*zerOS_rbtree_cmp_t)(struct zerOS_rbtree_head* head, void* data);

/**
 * User-provided augment function. Used to do recomputations when a child changed.
 */
typedef void (*zerOS_rbtree_augment_func_t)(struct zerOS_rbtree_head* head);

/**
 * Initialize an zerOS_rbtree_head.
 * After initialization, zerOS_rbtree_is_head_linked() will return false.
 */
static inline void zerOS_rbtree_reset_head(struct zerOS_rbtree_head* head)
{
	head->child[zerOS_RBTREE_LEFT]  = 0;
	head->child[zerOS_RBTREE_RIGHT] = 0;
	head->parent                    = 0;
}

/**
 * Initialize an zerOS_rbtree.
 */
static inline void zerOS_rbtree_reset_tree(struct zerOS_rbtree* tree)
{
	tree->base.child[zerOS_RBTREE_LEFT] = 0;
	/* ! see doc of zerOS_rbtree_is_base(). */
	tree->base.child[zerOS_RBTREE_RIGHT] = 3;
	tree->base.parent                    = 0;
}

/**
 * Get base head of tree.
 *
 * Warning: the base head is never embedded in a client payload structure.
 * It's just a link to host the real root of the tree as its left child.
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_base(struct zerOS_rbtree* tree)
{
	return &tree->base;
}

/**
 * Test if given head is base of tree.
 */
static inline bool zerOS_rbtree_is_base(struct zerOS_rbtree_head* head)
{
	/* We could check for the parent pointer being null, but by having
	 * a special sentinel right child value instead, we can make this
	 * function distinguish the base from unlinked pointers as well.
	 *
	 * A side effect is that this breaks programs with trees that are not
	 * initialized with zerOS_rbtree_init(), which could be a good or a bad thing,
	 * I don't know. */
	return head->child[zerOS_RBTREE_RIGHT] == 3;
}

/**
 * Check if a non-base head is linked in a (any) tree.
 */
static inline bool zerOS_rbtree_is_head_linked(struct zerOS_rbtree_head* head)
{
	return head->parent != 0;
}

/**
 * Get child in given direction, or NULL if there is no such child. `dir`
 * must be zerOS_RBTREE_LEFT or zerOS_RBTREE_RIGHT.
 */
static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_child(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return (struct zerOS_rbtree_head*)((head->child[dir]) & ~3);
}

/*
 * Test if a (left or right) child exists.
 * This is slightly more efficient than calling zerOS_rbtree_get_child() and comparing
 * to NULL.
 */
static inline bool
zerOS_rbtree_has_child(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return head->child[dir] != 0;
}

/**
 * Get direction from parent to child by testing the direction.
 *
 * Return zerOS_RBTREE_LEFT or zerOS_RBTREE_RIGHT, depending on whether this node is the left or
 * right child of its parent node. If the given node is the root node,
 * zerOS_RBTREE_LEFT is returned. (Technically the root node is the left child of the
 * base node).
 *
 * This is more convenient and (in theory) more efficient than getting the
 * parent and testing its left and right child.
 */
static inline enum zerOS_rbtree_direction
zerOS_rbtree_get_parent_dir(struct zerOS_rbtree_head* head)
{
	return head->parent & 1;
}

/**
 * Get parent head, or NULL if given node is the base head.
 *
 * Note that normally you don't want to visit the base head but stop already
 * at the root node.
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_parent(struct zerOS_rbtree_head* head)
{
	return (struct zerOS_rbtree_head*)(head->parent & ~3);
}

/**
 * Get topmost element of tree (or NULL if empty)
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_root(struct zerOS_rbtree* tree)
{
	return zerOS_rbtree_get_child(&tree->base, zerOS_RBTREE_LEFT);
}

/**
 * Check if tree is empty.
 */
static inline bool zerOS_rbtree_is_empty(struct zerOS_rbtree* tree)
{
	struct zerOS_rbtree_head* base = zerOS_rbtree_get_base(tree);
	return !zerOS_rbtree_has_child(base, zerOS_RBTREE_LEFT);
}

/**
 * Get minimum or maximum node in the tree, depending on the value of `dir`
 * (zerOS_RBTREE_LEFT or zerOS_RBTREE_RIGHT)
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head*
zerOS_rbtree_get_minmax(struct zerOS_rbtree* tree, enum zerOS_rbtree_direction dir);

/**
 * Get minimum (leftmost) element, or NULL if tree is empty.
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_min(struct zerOS_rbtree* tree)
{
	return zerOS_rbtree_get_minmax(tree, zerOS_RBTREE_LEFT);
}

/**
 * Get previous or next in-order descendant, depending on the value of `dir`
 * (zerOS_RBTREE_LEFT or zerOS_RBTREE_RIGHT).
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head* zerOS_rbtree_get_prevnext_descendant(
  struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir);

/**
 * Get previous or next in-order ancestor, depending on the value of `dir`
 * (zerOS_RBTREE_LEFT or zerOS_RBTREE_RIGHT).
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head*
zerOS_rbtree_get_prevnext_ancestor(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir);

/**
 * Get previous or next in-order node, depending on the value of `dir`.
 *
 * Time complexity: O(log n), amortized over sequential scan: O(1)
 */
extern struct zerOS_rbtree_head*
zerOS_rbtree_get_prevnext(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir);

/**
 * Get maximum (rightmost) element, or NULL if tree is empty
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_max(struct zerOS_rbtree* tree)
{
	return zerOS_rbtree_get_minmax(tree, zerOS_RBTREE_RIGHT);
}

/**
 * Get previous in-order node (maximal node in the tree that sorts before the
 * given element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n), amortized over sequential scan: O(1)
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_prev(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext(head, zerOS_RBTREE_LEFT);
}

/**
 * Get next in-order node (minimal node in the tree that sorts after the given
 * element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n), amortized over sequential scan: O(1)
 */
static inline struct zerOS_rbtree_head* zerOS_rbtree_get_next(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext(head, zerOS_RBTREE_RIGHT);
}

/**
 * Get previous in-order descendant (maximal descendant node that sorts before
 * the given element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_prev_descendant(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext_descendant(head, zerOS_RBTREE_LEFT);
}

/**
 * Get next in-order descendant (minimal descendant node that sorts after the
 * given element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_next_descendant(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext_descendant(head, zerOS_RBTREE_RIGHT);
}

/**
 * Get previous in-order ancestor (maximal ancestor node that sorts before the
 * given element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_prev_ancestor(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext_ancestor(head, zerOS_RBTREE_LEFT);
}

/**
 * Get next in-order ancestor (minimal ancestor node that sorts after the
 * given element) or NULL if no such element is in the tree.
 *
 * Time complexity: O(log n)
 */
static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_next_ancestor(struct zerOS_rbtree_head* head)
{
	return zerOS_rbtree_get_prevnext_ancestor(head, zerOS_RBTREE_RIGHT);
}

/**
 * Find a node in `tree` using `cmp` to direct the search. At each visited
 * node in the tree `cmp` is called with that node and `data` as arguments.
 * If a node that compares equal is found, it is returned. Otherwise, NULL is
 * returned.
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head*
zerOS_rbtree_find(struct zerOS_rbtree* tree, zerOS_rbtree_cmp_t cmp, void* data);

/**
 * Find a suitable insertion point for a new node in `tree` using `cmp` and
 * `data` to direct the search. At each visited node in the tree `cmp` is
 * called with that node and `data` as arguments. If a node that compares
 * equal is found, it is returned. Otherwise, NULL is returned and the
 * insertion point is returned as parent node and child direction in
 * `parent_out` and `dir_out`.
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head* zerOS_rbtree_find_parent(
  struct zerOS_rbtree*         tree,
  zerOS_rbtree_cmp_t           cmp,
  void*                        data,
  struct zerOS_rbtree_head**   parent_out,
  enum zerOS_rbtree_direction* dir_out);

/**
 * Link `head` into `tree` below another node in the given direction (zerOS_RBTREE_LEFT
 * or zerOS_RBTREE_RIGHT). The new node must replace a leaf. You can use
 * zerOS_rbtree_find_parent() to find the insertion point.
 *
 * `head` must not be linked into another tree when this function is called.
 *
 * Time complexity: O(log n)
 */
extern void zerOS_rbtree_link_and_rebalance(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   parent,
  enum zerOS_rbtree_direction dir);

/**
 * Unlink `head` from its current tree.
 *
 * Time complexity: O(log n)
 */
extern void zerOS_rbtree_unlink_and_rebalance(struct zerOS_rbtree_head* head);

/**
 * Replace `head` with `newhead`. `head` must be linked in a tree and
 * `newhead` must not be linked in a tree.
 */
extern void zerOS_rbtree_replace(struct zerOS_rbtree_head* head, struct zerOS_rbtree_head* newhead);

/**
 * Like zerOS_rbtree_link_and_rebalance(), but call an augmentation function for each
 * subtree that has been changed.
 */
extern void zerOS_rbtree_link_and_rebalance_and_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   parent,
  enum zerOS_rbtree_direction dir,
  zerOS_rbtree_augment_func_t augment);

/**
 * Like zerOS_rbtree_unlink_and_rebalance(), but call an augmentation function for each
 * subtree that has been changed.
 */
extern void zerOS_rbtree_unlink_and_rebalance_and_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment);

/**
 * Like zerOS_rbtree_replace(), but call an augmentation function for each subtree that has changed.
 */
extern void zerOS_rbtree_replace_and_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   newhead,
  zerOS_rbtree_augment_func_t augment);

/**
 * Update by calling the augmentation func for `head` and all its ancestors.
 */
extern void
zerOS_rbtree_update_augment(struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment);

/**
 * Find suitable insertion point for a new node in a subtree, directed by the
 * given search function. The subtree is given by its parent node `parent` and
 * child direction `dir`. The insertion point and its child direction are
 * returned in `parent_out` and `dir_out`.
 *
 * If the searched node is already in the tree (the compare function returns
 * 0), it is returned. In this case `parent_out` and `dir_out` are left
 * untouched. Otherwise NULL is returned.
 */
extern struct zerOS_rbtree_head* zerOS_rbtree_find_parent_in_subtree(
  struct zerOS_rbtree_head*    parent,
  enum zerOS_rbtree_direction  dir,
  zerOS_rbtree_cmp_t           cmp,
  void*                        data,
  struct zerOS_rbtree_head**   parent_out,
  enum zerOS_rbtree_direction* dir_out);

/**
 * Insert `head` into `tree` using `cmp` and `data` to direct the search. At
 * each visited node in the tree `cmp` is called with that node and `data` as
 * arguments (in that order). If a node that compares equal is found, it is
 * returned. Otherwise, `head` is inserted into the tree and NULL is
 * returned.
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head* zerOS_rbtree_insert(
  struct zerOS_rbtree* tree, struct zerOS_rbtree_head* head, zerOS_rbtree_cmp_t cmp, void* data);

/**
 * Find and delete a node from `tree` using `cmp` to direct the search. At
 * each visited node in the tree `cmp` is called with that node and `head` as
 * arguments (in that order). If a node that compares equal is found, it is
 * unlinked from the tree and returned. Otherwise, NULL is returned.
 *
 * Time complexity: O(log n)
 */
extern struct zerOS_rbtree_head*
zerOS_rbtree_delete(struct zerOS_rbtree* tree, zerOS_rbtree_cmp_t cmp, void* data);

/**
 * Given a node that is known to be linked in _some_ tree, find that tree.
 *
 * This involves a little hackery with offsetof(3)
 */
extern struct zerOS_rbtree* zerOS_rbtree_get_containing_tree(struct zerOS_rbtree_head* head);

/*
XXX: is inlining the search function advantageous?
*/
static inline struct zerOS_rbtree_head* zerOS_rbtree_INLINE_find(
  struct zerOS_rbtree_head*    parent,
  enum zerOS_rbtree_direction  dir,
  zerOS_rbtree_cmp_t           cmp,
  void*                        data,
  struct zerOS_rbtree_head**   parent_out,
  enum zerOS_rbtree_direction* dir_out)
{
	while (zerOS_rbtree_has_child(parent, dir))
	{
		parent = zerOS_rbtree_get_child(parent, dir);
		int r  = cmp(parent, data);
		if (r == 0)
			return parent;
		dir = (r < 0) ? zerOS_RBTREE_RIGHT : zerOS_RBTREE_LEFT;
	}
	if (parent_out)
		*parent_out = parent;
	if (dir_out)
		*dir_out = dir;
	return zerOS_RBTREE_NULL;
}

#undef zerOS_rbtree_head_to_container
#define zerOS_rbtree_head_to_container(container_type, pointer) \
	(container_of(container_type, pointer, head))

#if 0
/**************** DEBUG STUFF *******************/
int zerOS_rbtree_check_tree(struct zerOS_rbtree* tree);
/************************************************/
#endif

#ifdef __cplusplus
} // extern "C"
#endif

#endif
