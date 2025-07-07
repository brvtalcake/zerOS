/* rb3ptr  -- Intrusively linked 3-pointer Red-black tree implementation */

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

#include <stddef.h> // offsetof()
#include <zerOS/rbtree.h>

enum
{
	_RB3_DIR_BIT   = 1 << 0,
	_RB3_COLOR_BIT = 1 << 1,
	_RB3_BLACK     = 0,
	_RB3_RED       = _RB3_COLOR_BIT,
};

static inline zerOS_rbtree_ptr zerOS_rbtree_child_ptr(struct zerOS_rbtree_head* head, int color)
{
	return (zerOS_rbtree_ptr)head | color;
}

static inline zerOS_rbtree_ptr
zerOS_rbtree_parent_ptr(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return (zerOS_rbtree_ptr)head | dir;
}

static inline struct zerOS_rbtree_head*
zerOS_rbtree_get_black_child(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return (struct zerOS_rbtree_head*)head->child[dir];
}

static inline int
zerOS_rbtree_get_color_bit(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return head->child[dir] & _RB3_COLOR_BIT;
}

static inline bool
zerOS_rbtree_is_red(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return zerOS_rbtree_get_color_bit(head, dir) != 0;
}

static inline void
zerOS_rbtree_set_red(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	head->child[dir] |= _RB3_COLOR_BIT;
}

static inline void
zerOS_rbtree_set_black(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	head->child[dir] &= ~_RB3_COLOR_BIT;
}

static inline void zerOS_rbtree_connect(
  struct zerOS_rbtree_head*   head,
  enum zerOS_rbtree_direction dir,
  struct zerOS_rbtree_head*   child,
  int                         color)
{
	head->child[dir] = zerOS_rbtree_child_ptr(child, color);
	child->parent    = zerOS_rbtree_parent_ptr(head, dir);
}

static inline void zerOS_rbtree_connect_null(
  struct zerOS_rbtree_head*   head,
  enum zerOS_rbtree_direction dir,
  struct zerOS_rbtree_head*   child,
  int                         color)
{
	head->child[dir] = zerOS_rbtree_child_ptr(child, color);
	if (child)
		child->parent = zerOS_rbtree_parent_ptr(head, dir);
}

struct zerOS_rbtree* zerOS_rbtree_get_containing_tree(struct zerOS_rbtree_head* head)
{
	while (zerOS_rbtree_get_parent(head))
		head = zerOS_rbtree_get_parent(head);
	return (struct zerOS_rbtree*)((char*)head - (offsetof(struct zerOS_rbtree_head, child[0])));
}

static struct zerOS_rbtree_head*
zerOS_rbtree_get_minmax_in_subtree(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	if (!head)
		return zerOS_RBTREE_NULL;
	while (zerOS_rbtree_has_child(head, dir))
		head = zerOS_rbtree_get_child(head, dir);
	return head;
}

struct zerOS_rbtree_head*
zerOS_rbtree_get_minmax(struct zerOS_rbtree* tree, enum zerOS_rbtree_direction dir)
{
	return zerOS_rbtree_get_minmax_in_subtree(zerOS_rbtree_get_root(tree), dir);
}

struct zerOS_rbtree_head* zerOS_rbtree_get_prevnext_descendant(
  struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	return zerOS_rbtree_get_minmax_in_subtree(zerOS_rbtree_get_child(head, dir), !dir);
}

struct zerOS_rbtree_head*
zerOS_rbtree_get_prevnext_ancestor(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	/*
	 * Note: the direction is "reversed" for our purposes here, since
	 * the bit indicates the direction from the parent to `head`
	 */
	while (head && zerOS_rbtree_get_parent_dir(head) == dir)
	{
		head = zerOS_rbtree_get_parent(head);
	}
	if (head)
	{
		head = zerOS_rbtree_get_parent(head);
		if (!head || zerOS_rbtree_is_base(head))
			return zerOS_RBTREE_NULL;
		return head;
	}
	return zerOS_RBTREE_NULL;
}

struct zerOS_rbtree_head*
zerOS_rbtree_get_prevnext(struct zerOS_rbtree_head* head, enum zerOS_rbtree_direction dir)
{
	if (zerOS_rbtree_has_child(head, dir))
		return zerOS_rbtree_get_prevnext_descendant(head, dir);
	else
		return zerOS_rbtree_get_prevnext_ancestor(head, dir);
}

void zerOS_rbtree_update_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	while (!zerOS_rbtree_is_base(head))
	{
		augment(head);
		head = zerOS_rbtree_get_parent(head);
	}
}

static void zerOS_rbtree_rebalance_after_link(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	struct zerOS_rbtree_head*   pnt;
	struct zerOS_rbtree_head*   gpnt;
	struct zerOS_rbtree_head*   ggpnt;
	enum zerOS_rbtree_direction left;
	enum zerOS_rbtree_direction right;
	enum zerOS_rbtree_direction gdir;
	enum zerOS_rbtree_direction ggdir;

	if (!zerOS_rbtree_get_parent(zerOS_rbtree_get_parent(head)))
	{
		zerOS_rbtree_set_black(zerOS_rbtree_get_parent(head), zerOS_RBTREE_LEFT);
		if (augment)
			augment(head);
		return;
	}

	if (!zerOS_rbtree_is_red(
		  zerOS_rbtree_get_parent(zerOS_rbtree_get_parent(head)),
		  zerOS_rbtree_get_parent_dir(zerOS_rbtree_get_parent(head))))
	{
		/* parent is black */
		if (augment)
			zerOS_rbtree_update_augment(head, augment);
		return;
	}

	/*
	 * Since parent is red parent can't be the root.
	 * So we have at least a grandparent node, and grand-grandparent
	 * is either a real node or the base head.
	 */
	pnt   = zerOS_rbtree_get_parent(head);
	gpnt  = zerOS_rbtree_get_parent(pnt);
	ggpnt = zerOS_rbtree_get_parent(gpnt);
	left  = zerOS_rbtree_get_parent_dir(head);
	right = !zerOS_rbtree_get_parent_dir(head);
	gdir  = zerOS_rbtree_get_parent_dir(pnt);
	ggdir = zerOS_rbtree_get_parent_dir(gpnt);

	if (zerOS_rbtree_is_red(gpnt, !gdir))
	{
		/* uncle and parent are both red */
		zerOS_rbtree_set_red(ggpnt, ggdir);
		zerOS_rbtree_set_black(gpnt, zerOS_RBTREE_LEFT);
		zerOS_rbtree_set_black(gpnt, zerOS_RBTREE_RIGHT);
		if (augment)
			zerOS_rbtree_update_augment(head, augment);
		zerOS_rbtree_rebalance_after_link(gpnt, augment);
	}
	else if (gdir == right)
	{
		zerOS_rbtree_connect_null(pnt, left, zerOS_rbtree_get_black_child(head, right), _RB3_BLACK);
		zerOS_rbtree_connect_null(
		  gpnt, right, zerOS_rbtree_get_black_child(head, left), _RB3_BLACK);
		zerOS_rbtree_connect(head, left, gpnt, _RB3_RED);
		zerOS_rbtree_connect(head, right, pnt, _RB3_RED);
		zerOS_rbtree_connect(ggpnt, ggdir, head, _RB3_BLACK);
		if (augment)
		{
			augment(pnt);
			augment(gpnt);
			zerOS_rbtree_update_augment(head, augment);
		}
	}
	else
	{
		zerOS_rbtree_connect_null(gpnt, left, zerOS_rbtree_get_black_child(pnt, right), _RB3_BLACK);
		zerOS_rbtree_connect(pnt, right, gpnt, _RB3_RED);
		zerOS_rbtree_connect(ggpnt, ggdir, pnt, _RB3_BLACK);
		if (augment)
		{
			augment(gpnt);
			zerOS_rbtree_update_augment(head, augment);
		}
	}
}

static void zerOS_rbtree_rebalance_after_unlink(
  struct zerOS_rbtree_head*   pnt,
  enum zerOS_rbtree_direction pdir,
  zerOS_rbtree_augment_func_t augment)
{
	struct zerOS_rbtree_head*   gpnt;
	struct zerOS_rbtree_head*   sibling;
	struct zerOS_rbtree_head*   sleft;
	struct zerOS_rbtree_head*   sleftleft;
	struct zerOS_rbtree_head*   sleftright;
	enum zerOS_rbtree_direction left;
	enum zerOS_rbtree_direction right;
	enum zerOS_rbtree_direction gdir;

	if (!zerOS_rbtree_get_parent(pnt))
		return;

	left    = pdir; // define "left" as the direction from parent to deleted node
	right   = !pdir;
	gpnt    = zerOS_rbtree_get_parent(pnt);
	gdir    = zerOS_rbtree_get_parent_dir(pnt);
	sibling = zerOS_rbtree_get_child(pnt, right);
	sleft   = zerOS_rbtree_get_child(sibling, left);

	if (zerOS_rbtree_is_red(pnt, right))
	{
		/* sibling is red */
		zerOS_rbtree_connect(pnt, right, sleft, _RB3_BLACK);
		zerOS_rbtree_connect(sibling, left, pnt, _RB3_RED);
		zerOS_rbtree_connect(gpnt, gdir, sibling, _RB3_BLACK);
		if (augment)
			augment(sleft);
		zerOS_rbtree_rebalance_after_unlink(pnt, pdir, augment);
	}
	else if (zerOS_rbtree_is_red(sibling, right))
	{
		/* outer child of sibling is red */
		zerOS_rbtree_connect_null(pnt, right, sleft, zerOS_rbtree_get_color_bit(sibling, left));
		zerOS_rbtree_connect(sibling, left, pnt, _RB3_BLACK);
		zerOS_rbtree_connect(gpnt, gdir, sibling, zerOS_rbtree_get_color_bit(gpnt, gdir));
		if (augment)
		{
			zerOS_rbtree_update_augment(pnt, augment);
		}
		zerOS_rbtree_set_black(sibling, right);
	}
	else if (zerOS_rbtree_is_red(sibling, left))
	{
		/* inner child of sibling is red */
		sleftleft  = zerOS_rbtree_get_child(sleft, left);
		sleftright = zerOS_rbtree_get_child(sleft, right);
		zerOS_rbtree_connect_null(pnt, right, sleftleft, _RB3_BLACK);
		zerOS_rbtree_connect_null(sibling, left, sleftright, _RB3_BLACK);
		zerOS_rbtree_connect(sleft, left, pnt, _RB3_BLACK);
		zerOS_rbtree_connect(sleft, right, sibling, _RB3_BLACK);
		zerOS_rbtree_connect(gpnt, gdir, sleft, zerOS_rbtree_get_color_bit(gpnt, gdir));
		if (augment)
		{
			augment(sibling);
			zerOS_rbtree_update_augment(pnt, augment);
		}
	}
	else if (zerOS_rbtree_is_red(gpnt, gdir))
	{
		/* parent is red */
		zerOS_rbtree_set_red(pnt, right);
		zerOS_rbtree_set_black(gpnt, gdir);
		if (augment)
			zerOS_rbtree_update_augment(pnt, augment);
	}
	else
	{
		/* all relevant nodes are black */
		zerOS_rbtree_set_red(pnt, right);
		if (augment)
			augment(pnt);
		zerOS_rbtree_rebalance_after_unlink(gpnt, gdir, augment);
	}
}

void zerOS_rbtree_link_and_rebalance_and_maybe_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   parent,
  enum zerOS_rbtree_direction dir,
  zerOS_rbtree_augment_func_t augment)
{
	parent->child[dir]              = zerOS_rbtree_child_ptr(head, _RB3_RED);
	head->parent                    = zerOS_rbtree_parent_ptr(parent, dir);
	head->child[zerOS_RBTREE_LEFT]  = zerOS_rbtree_child_ptr(zerOS_RBTREE_NULL, _RB3_BLACK);
	head->child[zerOS_RBTREE_RIGHT] = zerOS_rbtree_child_ptr(zerOS_RBTREE_NULL, _RB3_BLACK);
	zerOS_rbtree_rebalance_after_link(head, augment);
}

void zerOS_rbtree_replace_and_maybe_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   newhead,
  zerOS_rbtree_augment_func_t augment)
{
	struct zerOS_rbtree_head*   left;
	struct zerOS_rbtree_head*   right;
	struct zerOS_rbtree_head*   parent;
	enum zerOS_rbtree_direction pdir;
	int                         pcol;

	*newhead = *head;

	left   = zerOS_rbtree_get_child(head, zerOS_RBTREE_LEFT);
	right  = zerOS_rbtree_get_child(head, zerOS_RBTREE_RIGHT);
	parent = zerOS_rbtree_get_parent(head);
	pdir   = zerOS_rbtree_get_parent_dir(head);
	pcol   = zerOS_rbtree_get_color_bit(parent, pdir);

	if (left)
		left->parent = zerOS_rbtree_parent_ptr(newhead, zerOS_RBTREE_LEFT);
	if (right)
		right->parent = zerOS_rbtree_parent_ptr(newhead, zerOS_RBTREE_RIGHT);
	parent->child[pdir] = zerOS_rbtree_child_ptr(newhead, pcol);

	if (augment)
		zerOS_rbtree_update_augment(newhead, augment);
}

static void zerOS_rbtree_unlink_noninternal_and_rebalance_and_maybe_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	struct zerOS_rbtree_head*   pnt;
	struct zerOS_rbtree_head*   cld;
	enum zerOS_rbtree_direction pdir;
	enum zerOS_rbtree_direction dir;

	dir = zerOS_rbtree_get_child(head, zerOS_RBTREE_RIGHT) ? zerOS_RBTREE_RIGHT : zerOS_RBTREE_LEFT;
	pnt = zerOS_rbtree_get_parent(head);
	cld = zerOS_rbtree_get_child(head, dir);
	pdir = zerOS_rbtree_get_parent_dir(head);

	bool must_rebalance = !zerOS_rbtree_is_red(pnt, pdir) && !zerOS_rbtree_is_red(head, dir);

	/* since we added the possibility for augmentation,
	we need to remove `head` *before* the rebalancing that we do below.
	(Otherwise the augmentation function would still see the to-be-deleted child). */
	zerOS_rbtree_connect_null(pnt, pdir, cld, _RB3_BLACK);

	if (must_rebalance)
		/* To be deleted node is black (and child cannot be repainted)
		 * => height decreased */
		zerOS_rbtree_rebalance_after_unlink(pnt, pdir, augment);
	else if (augment)
		/* the augment wasn't done since we didn't rebalance. So we need to do it separately.
		TODO: Could we restrict the augmentation done during rebalancing to just the
		nodes that aren't not be augmented by a regular zerOS_rbtree_augment_ancestors(pnt,
		augment)? */
		zerOS_rbtree_update_augment(pnt, augment);
}

static void zerOS_rbtree_unlink_internal_and_rebalance_and_maybe_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	struct zerOS_rbtree_head* subst;

	subst = zerOS_rbtree_get_next_descendant(head);
	zerOS_rbtree_unlink_noninternal_and_rebalance_and_maybe_augment(subst, augment);
	zerOS_rbtree_replace_and_maybe_augment(head, subst, augment);
}

void zerOS_rbtree_unlink_and_rebalance_and_maybe_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	if (
	  zerOS_rbtree_has_child(head, zerOS_RBTREE_LEFT)
	  && zerOS_rbtree_has_child(head, zerOS_RBTREE_RIGHT))
		zerOS_rbtree_unlink_internal_and_rebalance_and_maybe_augment(head, augment);
	else
		zerOS_rbtree_unlink_noninternal_and_rebalance_and_maybe_augment(head, augment);
}

struct zerOS_rbtree_head* zerOS_rbtree_find_parent_in_subtree(
  struct zerOS_rbtree_head*    parent,
  enum zerOS_rbtree_direction  dir,
  zerOS_rbtree_cmp_t           cmp,
  void*                        data,
  struct zerOS_rbtree_head**   parent_out,
  enum zerOS_rbtree_direction* dir_out)
{
	return zerOS_rbtree_INLINE_find(parent, dir, cmp, data, parent_out, dir_out);
}

struct zerOS_rbtree_head* zerOS_rbtree_insert(
  struct zerOS_rbtree* tree, struct zerOS_rbtree_head* head, zerOS_rbtree_cmp_t cmp, void* data)
{
	struct zerOS_rbtree_head*   found;
	struct zerOS_rbtree_head*   parent;
	enum zerOS_rbtree_direction dir;

	parent = zerOS_rbtree_get_base(tree);
	dir    = zerOS_RBTREE_LEFT;
	found  = zerOS_rbtree_find_parent_in_subtree(parent, dir, cmp, data, &parent, &dir);
	if (found)
		return found;
	zerOS_rbtree_link_and_rebalance(head, parent, dir);
	return zerOS_RBTREE_NULL;
}

struct zerOS_rbtree_head*
zerOS_rbtree_delete(struct zerOS_rbtree* tree, zerOS_rbtree_cmp_t cmp, void* data)
{
	struct zerOS_rbtree_head* found;

	found = zerOS_rbtree_find(tree, cmp, data);
	if (found)
	{
		zerOS_rbtree_unlink_and_rebalance(found);
		return found;
	}
	return zerOS_RBTREE_NULL;
}

struct zerOS_rbtree_head* zerOS_rbtree_find_parent(
  struct zerOS_rbtree*         tree,
  zerOS_rbtree_cmp_t           cmp,
  void*                        data,
  struct zerOS_rbtree_head**   parent_out,
  enum zerOS_rbtree_direction* dir_out)
{
	return zerOS_rbtree_find_parent_in_subtree(
	  zerOS_rbtree_get_base(tree), zerOS_RBTREE_LEFT, cmp, data, parent_out, dir_out);
}

struct zerOS_rbtree_head*
zerOS_rbtree_find(struct zerOS_rbtree* tree, zerOS_rbtree_cmp_t cmp, void* data)
{
	return zerOS_rbtree_find_parent_in_subtree(
	  zerOS_rbtree_get_base(tree), zerOS_RBTREE_LEFT, cmp, data, zerOS_RBTREE_NULL,
	  zerOS_RBTREE_NULL);
}

void zerOS_rbtree_link_and_rebalance(
  struct zerOS_rbtree_head* head, struct zerOS_rbtree_head* parent, enum zerOS_rbtree_direction dir)
{
	zerOS_rbtree_link_and_rebalance_and_maybe_augment(head, parent, dir, zerOS_RBTREE_NULL);
}

void zerOS_rbtree_unlink_and_rebalance(struct zerOS_rbtree_head* head)
{
	zerOS_rbtree_unlink_and_rebalance_and_maybe_augment(head, zerOS_RBTREE_NULL);
}

void zerOS_rbtree_replace(struct zerOS_rbtree_head* head, struct zerOS_rbtree_head* newhead)
{
	zerOS_rbtree_replace_and_maybe_augment(head, newhead, zerOS_RBTREE_NULL);
}

void zerOS_rbtree_link_and_rebalance_and_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   parent,
  enum zerOS_rbtree_direction dir,
  zerOS_rbtree_augment_func_t augment)
{
	zerOS_rbtree_link_and_rebalance_and_maybe_augment(head, parent, dir, augment);
}

void zerOS_rbtree_unlink_and_rebalance_and_augment(
  struct zerOS_rbtree_head* head, zerOS_rbtree_augment_func_t augment)
{
	zerOS_rbtree_unlink_and_rebalance_and_maybe_augment(head, augment);
}

void zerOS_rbtree_replace_and_augment(
  struct zerOS_rbtree_head*   head,
  struct zerOS_rbtree_head*   newhead,
  zerOS_rbtree_augment_func_t augment)
{
	zerOS_rbtree_replace_and_maybe_augment(head, newhead, augment);
}

#if 0
/* DEBUG STUFF */

	#include <stdio.h>
static void visit_inorder_helper(struct zerOS_rbtree_head* head, int isred)
{
	if (!head)
		return;
	printf(" (");
	visit_inorder_helper(
	  zerOS_rbtree_get_child(head, zerOS_RBTREE_LEFT),
	  zerOS_rbtree_is_red(head, zerOS_RBTREE_LEFT));
	printf("%s", isred ? "R" : "B");
	visit_inorder_helper(
	  zerOS_rbtree_get_child(head, zerOS_RBTREE_RIGHT),
	  zerOS_rbtree_is_red(head, zerOS_RBTREE_RIGHT));
	printf(")");
}

static void visit_inorder(struct zerOS_rbtree* tree)
{
	visit_inorder_helper(zerOS_rbtree_get_root(tree), 0);
	printf("\n");
}

static int
zerOS_rbtree_is_valid_tree_helper(struct zerOS_rbtree_head* head, int isred, enum zerOS_rbtree_direction dir, int* depth)
{
	int i;
	int depths[2] = { 1, 1 };

	*depth = 1;

	if (!head)
	{
		if (isred)
		{
			printf("red leaf child!\n");
			return 0;
		}
		return 1;
	}

	if (zerOS_rbtree_get_parent_dir(head) != dir)
	{
		printf("Directions messed up!\n");
		return 0;
	}

	for (i = 0; i < 2; i++)
	{
		if (isred && zerOS_rbtree_get_color_bit(head, i))
		{
			printf("two red in a row!\n");
			return 0;
		}
		if (!zerOS_rbtree_is_valid_tree_helper(
			  zerOS_rbtree_get_child(head, i), zerOS_rbtree_is_red(head, i), i, &depths[i]))
			return 0;
	}
	if (depths[0] != depths[1])
	{
		printf("Unbalanced tree! got %d and %d\n", depths[0], depths[1]);
		return 0;
	}
	*depth = depths[0] + !isred;

	return 1;
}

int zerOS_rbtree_check_tree(struct zerOS_rbtree* tree)
{
	int depth;
	int valid;

	if (zerOS_rbtree_is_red(&tree->base, zerOS_RBTREE_LEFT))
	{
		printf("Error! root is red.\n");
		return 0;
	}

	valid = zerOS_rbtree_is_valid_tree_helper(zerOS_rbtree_get_root(tree), 0, 0, &depth);
	if (!valid)
		visit_inorder(tree);
	return valid;
}
#endif
