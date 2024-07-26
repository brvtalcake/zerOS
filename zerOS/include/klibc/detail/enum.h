#ifndef zerOS_KLIBC_DETAIL_ENUM_H_INCLUDED
#define zerOS_KLIBC_DETAIL_ENUM_H_INCLUDED

#undef  UNDERLYING_TYPE
/**
 * @def UNDERLYING_TYPE(type)
 * @brief A macro that defines the underlying type of an enumeration.
 * @details This macro is used to define the underlying type of an enumeration.
 *          When __INTELLISENSE__ is defined, the macro expands to nothing, and
 *          otherwise expands to C23's `: type`.
 * @param type The type of the enumeration.
 */
#ifdef __INTELLISENSE__
    #define UNDERLYING_TYPE(type)
#else
    #define UNDERLYING_TYPE(type) : type
#endif

#endif
