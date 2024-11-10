#ifndef zerOS_KLIBC_MAYBE_H_INCLUDED
#define zerOS_KLIBC_MAYBE_H_INCLUDED

#include <kernel/compiler/cast.h>

#undef  maybe_type
/**
 * @def maybe_type(type)
 * @brief Defines a `maybe` type.
 * @param type The type of the `maybe`.
 */
#define maybe_type(type)    \
    struct {                \
        typeof_unqual(type) \
          value;            \
        bool valid;         \
    }

#undef  maybe_some
/**
 * @def maybe_some(value)
 * @brief Creates a `maybe` with a value.
 * @param type The type of the `maybe`.
 * @param val The value to store.
 */
#define maybe_some(type, val) \
    ((maybe_type(type)) { .value = reinterpret_cast(type, val), .valid = true })

#undef  maybe_none
/**
 * @def maybe_none
 * @brief Creates a `maybe` without a value.
 * @param type The type of the `maybe`.
 */
#define maybe_none(type) \
    ((maybe_type(type)) { .valid = false })

#undef  maybe_is_some
/**
 * @def maybe_is_some(maybe)
 * @brief Checks if a `maybe` has a value.
 * @param maybe The `maybe` to check.
 */
#define maybe_is_some(maybe) \
    ((maybe).valid)

#undef  maybe_is_none
/**
 * @def maybe_is_none(maybe)
 * @brief Checks if a `maybe` does not have a value.
 * @param maybe The `maybe` to check.
 */
#define maybe_is_none(maybe) \
    (!maybe_is_some(maybe))

#undef  maybe_unwrap
/**
 * @def maybe_unwrap(maybe)
 * @brief Unwraps a `maybe`.
 * @param maybe The `maybe` to unwrap.
 */
#define maybe_unwrap(maybe) \
    ((maybe).value)

#undef  maybe_unwrap_or
/**
 * @def maybe_unwrap_or(maybe, default)
 * @brief Unwraps a `maybe` or returns a default value.
 * @param maybe The `maybe` to unwrap.
 * @param default The default value to return.
 */
#define maybe_unwrap_or(maybe, default) \
    (maybe_is_some(maybe) ? maybe_unwrap(maybe) : (default))

#endif
