#ifndef zerOS_KLIBC_SETJMP_H_INCLUDED
#define zerOS_KLIBC_SETJMP_H_INCLUDED

#ifdef __has_include_next
    #if __has_include_next(<setjmp.h>)
        #include_next <setjmp.h>
    #else
        #undef  __KLIBC_NEED_SETJMP_DEFS
        #define __KLIBC_NEED_SETJMP_DEFS 1
    #endif
#else
    #undef  __KLIBC_NEED_SETJMP_DEFS
    #define __KLIBC_NEED_SETJMP_DEFS 1
#endif

#if __KLIBC_NEED_SETJMP_DEFS

// TODO: Implement setjmp.h
//   either with gcc's built-in setjmp/longjmp, or
//   with a custom implementation in assembly

#endif

#undef  __KLIBC_NEED_SETJMP_DEFS
#endif
