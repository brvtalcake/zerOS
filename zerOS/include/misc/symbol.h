#ifndef zerOS_MISC_SYMBOL_H_INCLUDED
#define zerOS_MISC_SYMBOL_H_INCLUDED

#undef  SYMBOL_USED
/**
 * @def SYMBOL_USED
 * @brief Marks a symbol as being used.
 */
#define SYMBOL_USED [[__gnu__::__used__]]

#undef  SYMBOL_ALIGNED
/**
 * @def SYMBOL_ALIGNED
 * @brief Aligns a symbol to a specific alignment.
 */
#define SYMBOL_ALIGNED(align) [[__gnu__::__aligned__(align)]]

#endif
