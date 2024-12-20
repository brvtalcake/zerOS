#ifndef zerOS_MISC_SYMBOL_H_INCLUDED
#define zerOS_MISC_SYMBOL_H_INCLUDED

#undef SYMBOL_USED
/**
 * @def SYMBOL_USED
 * @brief Marks a symbol as being used.
 */
#define SYMBOL_USED [[__gnu__::__used__]]

#undef SYMBOL_UNUSED
/**
 * @def SYMBOL_UNUSED
 * @brief Marks a symbol as being unused.
 */
#ifdef __has_c_attribute
    #if __has_c_attribute(__unused__) || __has_c_attribute(__maybe_unused__)
        #define SYMBOL_UNUSED [[__maybe_unused__]]
    #endif
#endif

#ifndef SYMBOL_UNUSED
    #define SYMBOL_UNUSED [[__gnu__::__unused__]]
#endif

#undef SYMBOL_ALIGNED_TO
/**
 * @def SYMBOL_ALIGNED_TO
 * @brief Aligns a symbol to a specific alignment.
 */
#define SYMBOL_ALIGNED_TO(align) [[__gnu__::__aligned__(align)]]

/**
 * @typedef symbol
 * @brief A symbol, as the ones exposed by the linker.
 */
typedef unsigned char symbol[];

#undef SYMBOL_COUNTED_BY
/**
 * @def SYMBOL_COUNTED_BY
 * @brief Marks a structure field as being counted by another field.
 */
#define SYMBOL_COUNTED_BY(field) [[__gnu__::__counted_by__(field)]]

#endif
