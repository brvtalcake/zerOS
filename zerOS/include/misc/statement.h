#ifndef zerOS_MISC_STATEMENT_H_INCLUDED
#define zerOS_MISC_STATEMENT_H_INCLUDED

#undef  CASE_FALLTHROUGH
/**
 * @def CASE_FALLTHROUGH
 * @brief Specify that a `case` statement should fall through to the next case statement
 * without issuing a warning.
 */
#define CASE_FALLTHROUGH [[__gnu__::__fallthrough__]]

#endif
