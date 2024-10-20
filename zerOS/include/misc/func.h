#ifndef zerOS_MISC_FUNC_H_INCLUDED
#define zerOS_MISC_FUNC_H_INCLUDED

#undef  FUNC_NORETURN
/**
 * @def FUNC_NORETURN
 * @brief Marks a function as not returning.
 */
#define FUNC_NORETURN [[__noreturn__]]

#endif
