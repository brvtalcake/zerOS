#include <config.h>

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#include <cpuid.h>

#include <kernel/cpu/io.h>
#include <kernel/cpu/cpu.h>

#include <misc/sections.h>

extern void zerOS_cpuid_count(uint32_t leaf, uint32_t subleaf, struct zerOS_cpuid_info* info)
{
    __cpuid_count(leaf, subleaf, info->eax, info->ebx, info->ecx, info->edx);
}

extern void zerOS_cpuid(uint32_t leaf, struct zerOS_cpuid_info* info)
{
    zerOS_cpuid_count(leaf, 0, info);
}

extern void zerOS_set_ia32_misc(bool value, uint8_t bit)
{
    // Set the IA32_MISC_ENABLE MSR bit to the specified value
    uint64_t reg = zerOS_read_msr(0x1A0);
    if (value)
        reg |= (1 << bit);
    else
        reg &= ~(1 << bit);
    zerOS_write_msr(0x1A0, reg);
}


