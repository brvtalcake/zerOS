// All functions here are following SysV ABI calling convention

#include <asm/gas/common.inc>

#include <asm/common.h>
#include <asm/syntax.h>

#include <chaos/preprocessor.h>

#undef  __digit_chars_lower
#undef  __digit_chars_upper
#define __digit_chars_lower (0)(1)(2)(3)(4)(5)(6)(7)(8)(9)(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)(m)(n)(o)(p)(q)(r)(s)(t)(u)(v)(w)(x)(y)(z)
#define __digit_chars_upper (0)(1)(2)(3)(4)(5)(6)(7)(8)(9)(A)(B)(C)(D)(E)(F)(G)(H)(I)(J)(K)(L)(M)(N)(O)(P)(Q)(R)(S)(T)(U)(V)(W)(X)(Y)(Z)

#undef  __digits_table_build
#define __digits_table_build(syms)  \
    CHAOS_PP_EXPR(                  \
        CHAOS_PP_SEQ /* TODO */

IN_SECTION(.rodata)
__digits_lower __digits_table_build(__digit_chars_lower)
    

IN_SECTION(.bootcode)

extern_func __early_printk_u64_to_string
// TODO: Implement this function

