#ifndef zerOS_KERNEL_QEMU_H_INCLUDED
#define zerOS_KERNEL_QEMU_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <kernel/cpu/cpu.h>

// TODO: optimize so we don't have to call this function every time
static inline bool zerOS_in_qemu(void)
{
    struct zerOS_cpuid_info info;
    zerOS_cpuid(0x40000000, &info);
    
    char hyperv[12];
    *(uint32_t*)&hyperv[0] = info.ebx;
    *(uint32_t*)&hyperv[4] = info.ecx;
    *(uint32_t*)&hyperv[8] = info.edx;
    
    bool is_qemu = true;
    char qemu_hyperv[2][12] = {
        "TCGTCGTCGTCG",
        "KVMKVMKVM\0\0\0"
    };

    for (size_t i = 0; is_qemu && i < 12; ++i)
        is_qemu &= hyperv[i] == qemu_hyperv[0][i];

    if (is_qemu)
        return true;
    
    is_qemu = true;
    for (size_t i = 0; is_qemu && i < 12; ++i)
        is_qemu &= hyperv[i] == qemu_hyperv[1][i];
    
    return is_qemu;
}

#endif
