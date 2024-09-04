#ifndef zerOS_KLIBC_DETAIL_FRAME_INFO_H_INCLUDED
#define zerOS_KLIBC_DETAIL_FRAME_INFO_H_INCLUDED

#include <stdbool.h>
#include <klibc/generated/sections.h>
#include <klibc/preprocessor/default_arg.h>

#undef  KLIBC_IN_SECTION
/**
 * @def KLIBC_IN_SECTION(addr, section_name)
 * @brief Check whether an address is in a specific section.
 * @param addr         The address to check.
 * @param section_name The name of the section.
 */
#define KLIBC_IN_SECTION(addr, section_name)                                    \
    ((bool)((uintptr_t)(addr) >= (uintptr_t)(zerOS_##section_name##_start) &&   \
            (uintptr_t)(addr) <  (uintptr_t)(zerOS_##section_name##_end)))

#undef  KLIBC_CALLER_ADDR
/**
 * @def KLIBC_CALLER_ADDR(depth)
 * @brief Get the address of the caller at a specific depth.
 * @param depth The depth of the caller.
 */
#define KLIBC_CALLER_ADDR(...) __builtin_extract_return_addr(__builtin_return_address(KLIBC_PP_DEFAULT_ARG(1U, __VA_ARGS__)))

#undef  KLIBC_CALLER_IN_SECTION
/**
 * @def KLIBC_CALLER_IN_SECTION(depth, section_name)
 * @brief Check whether the caller at a specific depth is in a specific section.
 * @param section_name The name of the section.
 * @param depth        (optional) The depth of the caller.
 */
#define KLIBC_CALLER_IN_SECTION(section_name, ...) KLIBC_IN_SECTION(KLIBC_CALLER_ADDR(__VA_ARGS__), section_name)
 

#endif
