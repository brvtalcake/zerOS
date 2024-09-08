#ifndef zerOS_KERNEL_MEMORY_ALIGN_H_INCLUDED
#define zerOS_KERNEL_MEMORY_ALIGN_H_INCLUDED

#undef  ALIGN_UP
/**
 * @def ALIGN_UP(x, align)
 * @brief Aligns a value up to the specified alignment.
 * @param x The value to align.
 * @param align The alignment to use.
 */
#define ALIGN_UP(x, align) (((x) + (align) - 1) & ~((align) - 1))

#endif
