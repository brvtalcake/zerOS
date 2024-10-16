#ifndef zerOS_ASM_SYNTAX_H_INCLUDED
#define zerOS_ASM_SYNTAX_H_INCLUDED

#ifdef __ASSEMBLER__

#include <chaos/preprocessor/extended/variadic_cat.h>

#include <pp_empty/pp_is_empty.h>

#undef  ASM_SYNTAX_INTEL
#undef  ASM_SYNTAX_ATT

#define ASM_SYNTAX_INTEL 1
#define ASM_SYNTAX_ATT   2

#undef  __ASM_SYNTAX_UNDEFINED_IMPL
#define __ASM_SYNTAX_UNDEFINED_IMPL

#undef  ASM_SYNTAX_UNDEFINED
#define ASM_SYNTAX_UNDEFINED ISEMPTY(CHAOS_PP_VARIADIC_CAT(__, ASM_SYNTAX, _UNDEFINED_IMPL))

#endif

#endif
