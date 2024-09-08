#ifndef zerOS_ASSERT_H_INCLUDED
#define zerOS_ASSERT_H_INCLUDED

#include <kernel/cpu/misc.h>

#undef  static_assert
/**
 * @def static_assert
 * @brief Statically sserts that a condition is true.
 * 
 * @param __VA_ARGS__ The expression to be evaluated.
 * 
 */
#define static_assert(...) static_assert(__VA_ARGS__)

#undef  assert
/**
 * @def assert
 * @brief Asserts that a condition is true.
 * 
 * @param __VA_ARGS__ The expression to be evaluated.
 * @todo For now, we just "hcf", but we'll have to update it when able to print to fb.
 * 
 */
#define assert(cond, ...) do { if (!(cond)) zerOS_hcf(); }

#endif
