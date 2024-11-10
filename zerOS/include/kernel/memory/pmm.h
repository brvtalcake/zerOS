#ifndef zerOS_KERNEL_MEMORY_PMM_H_INCLUDED
#define zerOS_KERNEL_MEMORY_PMM_H_INCLUDED

#include <klibc/maybe.h>

// clang-format off
struct zerOS_address
{
    uintptr_t phys,
              virt;
};
// clang-format on

extern bool zerOS_init_pmm(void);

#endif
