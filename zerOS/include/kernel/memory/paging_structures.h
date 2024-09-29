#ifndef zerOS_KERNEL_MEMORY_PAGING_STRUCTURES_H_INCLUDED
#define zerOS_KERNEL_MEMORY_PAGING_STRUCTURES_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <misc/type.h>

#include <kernel/compiler/bitfield.h>

struct TYPE_PACKED zerOS_pml4_entry
{
    BITFIELD_VALUE(present, 1);
    BITFIELD_VALUE(read_write, 1);
    BITFIELD_VALUE(user_supervisor, 1);
    BITFIELD_VALUE(write_through, 1);
    BITFIELD_VALUE(cache_disabled, 1);
    BITFIELD_VALUE(accessed, 1);
    BITFIELD_VALUE(_ignored1, 1);
    BITFIELD_VALUE(_mbz, 2);
    BITFIELD_VALUE(avail, 3);
    BITFIELD_VALUE(pdp_base_address, 40);
    BITFIELD_VALUE(_ignored2, 11);
    BITFIELD_VALUE(execute_disable, 1);
};

static_assert(
    sizeof(struct zerOS_pml4_entry) * 8 == 64,
    "struct zerOS_pml4_entry is not 64 bits wide"
);

struct TYPE_PACKED zerOS_pdp_entry
{
    BITFIELD_VALUE(present, 1);
    BITFIELD_VALUE(read_write, 1);
    BITFIELD_VALUE(user_supervisor, 1);
    BITFIELD_VALUE(write_through, 1);
    BITFIELD_VALUE(cache_disabled, 1);
    BITFIELD_VALUE(accessed, 1);
    BITFIELD_VALUE(_ignored1, 1);
    BITFIELD_VALUE(_mbz, 1);
    BITFIELD_VALUE(avail, 3);
    BITFIELD_VALUE(pd_base_address, 40);
    BITFIELD_VALUE(_ignored2, 11);
    BITFIELD_VALUE(execute_disable, 1);
};

static_assert(
    sizeof(struct zerOS_pdp_entry) * 8 == 64,
    "struct zerOS_pdp_entry is not 64 bits wide"
);

#endif
