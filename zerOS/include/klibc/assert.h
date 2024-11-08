#include <klibc/preprocessor/default_arg.h>
#include <klibc/preprocessor/variadics.h>

#include <stddef.h>
#include <stdbool.h>

#undef  hard_assert
/**
 * @def hard_assert(cond, ...)
 * @brief Asserts a condition at run-time.
 * @param cond The condition.
 * @param ...  The message, if needed.
 */
#define hard_assert(cond, ...)              \
    do {                                    \
        if (unlikely(!(cond)))              \
        {                                   \
            KLIBC_HARD_ASSERT_HOOK(         \
                KLIBC_PP_DEFAULT_ARG(       \
                    #cond,                  \
                    __VA_ARGS__             \
                ),                          \
                __FILE__,                   \
                __LINE__,                   \
                __PRETTY_FUNCTION__         \
            );                              \
        }                                   \
    } while (false)
