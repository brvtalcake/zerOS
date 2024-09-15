#ifndef zerOS_MISC_UNIQUE_IDENT_H_INCLUDED
#define zerOS_MISC_UNIQUE_IDENT_H_INCLUDED

#include <chaos/preprocessor/extended/variadic_cat.h>

#undef  UNIQUE
/**
 * @def UNIQUE
 * @brief Generate a unique identifier for use in a macro definition.
 */
#define UNIQUE(...) CHAOS_PP_VARIADIC_CAT(___uNiQuE_iDeNtIfIeR_at_LINE, __LINE__, _NAMED, __VA_ARGS__)

#endif

