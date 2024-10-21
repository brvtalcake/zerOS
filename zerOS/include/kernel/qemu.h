#ifndef zerOS_KERNEL_QEMU_H_INCLUDED
#define zerOS_KERNEL_QEMU_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <kernel/cpu/cpu.h>

static inline bool zerOS_in_qemu(void)
{
    struct zerOS_cpuid_info info;
    if (!zerOS_cpuid(0x40000000, &info))
        return false;
    
    char hyperv[12];
    *(uint32_t*)&hyperv[0] = info.ebx;
    *(uint32_t*)&hyperv[4] = info.ecx;
    *(uint32_t*)&hyperv[8] = info.edx;
    
    bool is_qemu = true;
    char qemu_hyperv[2][4] = {
        {'T', 'C', 'G', '\0'},
        {'K', 'V', 'M', '\0'}
    };

    for (size_t i = 0; is_qemu && i < 12; i += 4)
        is_qemu &= hyperv[i    ] == qemu_hyperv[0][i / 4    ] &&
                   hyperv[i + 1] == qemu_hyperv[0][i / 4 + 1] &&
                   hyperv[i + 2] == qemu_hyperv[0][i / 4 + 2] &&
                   hyperv[i + 3] == qemu_hyperv[0][i / 4 + 3];

    if (is_qemu)
        return true;
    
    is_qemu = true;
    for (size_t i = 0; is_qemu && i < 12; i += 4)
        is_qemu &= hyperv[i    ] == qemu_hyperv[1][i / 4    ] &&
                   hyperv[i + 1] == qemu_hyperv[1][i / 4 + 1] &&
                   hyperv[i + 2] == qemu_hyperv[1][i / 4 + 2] &&
                   hyperv[i + 3] == qemu_hyperv[1][i / 4 + 3];
    
    return is_qemu;
}

#endif
