#ifndef zerOS_STDLIB_H_INCLUDED
#define zerOS_STDLIB_H_INCLUDED

#undef _GOT_REAL_STDLIB_H_

#if defined(__has_include_next)
    #if __has_include_next(<stdlib.h>)
        #include_next <stdlib.h>
        #define _GOT_REAL_STDLIB_H_
    #endif
#endif

#ifndef _GOT_REAL_STDLIB_H_
    #include <klibc/stdlib.h>
#else
    #undef _GOT_REAL_STDLIB_H_
#endif

#endif
