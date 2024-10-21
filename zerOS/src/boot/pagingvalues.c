#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <kernel/printk.h>
#include <kernel/cpu/cpu.h>
#include <kernel/cpu/misc.h>

#include <klibc/preprocessor/binary.h>

#include <misc/sections.h>

size_t zerOS_maxphyaddr;
uint64_t zerOS_pagetable_phyaddr_mask[3];

BOOT_FUNC
static inline void get_maxphyaddr(void)
{
    struct zerOS_cpuid_info info;
    if (!zerOS_cpuid(0x80000008, &info))
        zerOS_hcf();
    zerOS_maxphyaddr = info.eax & 0xFF;
}

BOOT_FUNC
static inline void set_pagetable_phyaddr_masks(void)
{
#if !defined __INTELLISENSE__
    zerOS_pagetable_phyaddr_mask[0] = KLIBC_PP_BINARY_DEMOTE(
        KLIBC_PP_BINARY(
            (0, (0 , 11)),
            (1, (12, 51)),
            (0, (52, 63))
        )
    );
    zerOS_pagetable_phyaddr_mask[1] = KLIBC_PP_BINARY_DEMOTE(
        KLIBC_PP_BINARY(
            (0, (0 , 20)),
            (1, (21, 51)),
            (0, (52, 63))
        )
    );
    zerOS_pagetable_phyaddr_mask[2] = KLIBC_PP_BINARY_DEMOTE(
        KLIBC_PP_BINARY(
            (0, (0 , 29)),
            (1, (30, 51)),
            (0, (52, 63))
        )
    );
#endif
}

BOOT_FUNC
void zerOS_init_paging_values(void)
{
    get_maxphyaddr();
    zerOS_early_printk("zerOS: MAXPHYADDR = %u\n", (unsigned int) zerOS_maxphyaddr);
    set_pagetable_phyaddr_masks();
}