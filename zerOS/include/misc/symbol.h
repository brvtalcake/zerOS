#ifndef zerOS_MISC_SYMBOL_H_INCLUDED
#define zerOS_MISC_SYMBOL_H_INCLUDED

#undef  SYMBOL_USED
/**
 * @def SYMBOL_USED
 * @brief Marks a symbol as being used.
 */
#define SYMBOL_USED [[__gnu__::__used__]]

#undef  SYMBOL_ALIGNED_TO
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

#undef  SYMBOL_COUNTED_BY
/**
 * @def SYMBOL_COUNTED_BY
 * @brief Marks a structure field as being counted by another field.
 */
#define SYMBOL_COUNTED_BY(field) [[__gnu__::__counted_by__(field)]]

#endif
