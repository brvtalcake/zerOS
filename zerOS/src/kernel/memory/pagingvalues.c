#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <klibc/preprocessor/binary.h>

#include <misc/sections.h>

size_t zerOS_maxphyaddr;
uint64_t zerOS_pagetable_phyaddr_mask[3];

BOOT_FUNC
void zerOS_init_paging_values(void)
{
#ifndef __INTELLISENSE__
    KLIBC_PP_BINARY((0, (0 , 11)),(1, (12, 51)),(0, (52, 63)))
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