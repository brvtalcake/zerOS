#ifndef zerOS_MISC_PRINTF_H_INCLUDED
#define zerOS_MISC_PRINTF_H_INCLUDED

#undef  PRINTF_LIKE
/**
 * @def PRINTF_LIKE
 * @brief Marks a function as having printf-like semantics.
 * @param fmt_arg The format string argument index.
 * @param var_arg The first variable argument index.
 */
#define PRINTF_LIKE(fmt_arg, var_arg) [[__gnu__::__format__(__printf__, fmt_arg, var_arg)]]

#undef  VPRINTF_LIKE
/**
 * @def VPRINTF_LIKE
 * @brief Marks a function as having vprintf-like semantics.
 * @param fmt_arg The format string argument index.
 */
#define VPRINTF_LIKE(fmt_arg) [[__gnu__::__format_arg__(fmt_arg)]]

#endif
