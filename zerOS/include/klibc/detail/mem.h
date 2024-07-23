#ifndef zerOS_KLIBC_DETAIL_MEM_H_INCLUDED
#define zerOS_KLIBC_DETAIL_MEM_H_INCLUDED

#undef  KLIBC_ALIGN_UP
/**
 * @def KLIBC_ALIGN_UP(x, align)
 * @brief Aligns a value up to the specified alignment.
 * @param x The value to align.
 * @param align The alignment to use.
 */
#define KLIBC_ALIGN_UP(x, align) (((x) + (align) - 1) & ~((align) - 1))

#endif
