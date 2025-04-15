// All functions here are following SysV ABI calling convention

#include <stdint.h>

#undef  __MK_ABS
#define __MK_ABS(typesize)                                  \
    extern uint##typesize##_t abs##typesize                 \
        (int##typesize##_t v)                               \
    {                                                       \
        uint##typesize##_t r;                               \
        const int##typesize##_t mask = v >> (               \
            sizeof(int##typesize##_t) * __CHAR_BIT__ - 1    \
        );                                                  \
        r = (v ^ mask) - mask;                              \
        return r;                                           \
    }

__MK_ABS(8)
__MK_ABS(16)
__MK_ABS(32)
__MK_ABS(64)
__MK_ABS(128)