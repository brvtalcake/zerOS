#ifndef zerOS_KERNEL_QEMU_H_INCLUDED
#define zerOS_KERNEL_QEMU_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

static inline bool zerOS_in_qemu(void)
{
    uint32_t eax, ebx, ecx, edx;
    zerOS_cpuid(0, &eax, &ebx, &ecx, &edx);
    return ebx == 0x51434D55 && ecx == 0x4D566572 && edx == 0x65766949;
}

#endif
