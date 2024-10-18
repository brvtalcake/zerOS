#include <chaos/preprocessor.h>

#define A(n) \
    CHAOS_PP_WHEN(n)( \
        A_INDIRECT CHAOS_PP_OBSTRUCT()()(CHAOS_PP_DEC(n)) \
        n \
    ) \
    /**/
#define A_INDIRECT() A

#define I(x) x

( I(I(I(I(I( A(3) ))))) ) // ( 1 2 3 )