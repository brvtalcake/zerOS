#ifndef zerOS_ATOMICS_H_INCLUDED_
#define zerOS_ATOMICS_H_INCLUDED_ 1

#include <stdint.h>
#include <x86intrin.h>
#include <zerOS/common.h>

// TODO: implement my own atomics (see `libatomic_ops` for inspiration)
// TODO: for x86/amd64, use a combination of lock prefix (when needed ?), HLE/XBEGIN/XEND on Intel,
// and maybe something else for AMD processors ?

// static_assert(__atomic_always_lock_free(sizeof(uint32_t), nullptr));
//
// struct zerOS_atomic_flag
//{
//     uint32_t flag;
// };
//
// #undef zerOS_ATOMIC_FLAG_INITIALIZER
// #define zerOS_ATOMIC_FLAG_INITIALIZER { .flag = 0 }

#endif
