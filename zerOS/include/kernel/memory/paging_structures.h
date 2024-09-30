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
    BITFIELD_VALUE(_reserved, 3); // Must be 0
    BITFIELD_VALUE(_avail1, 3); // Available for use by the OS
    BITFIELD_VALUE(pdp_base_address, 40);
    BITFIELD_VALUE(_avail2, 11); // Available for use by the OS
    BITFIELD_VALUE(execute_disable, 1);
};

static_assert(
    sizeof(struct zerOS_pml4_entry) * 8 == 64,
    "struct zerOS_pml4_entry is not 64 bits wide"
);
static_assert(
    1 + 1 + 1 + 1 + 1 + 1 + 3 + 3 + 40 + 11 + 1 == 64,
    "struct zerOS_pml4_entry bitfield values do not add up to 64"
);

struct TYPE_PACKED zerOS_pdp_entry
{
    BITFIELD_VALUE(present, 1);
    BITFIELD_VALUE(read_write, 1);
    BITFIELD_VALUE(user_supervisor, 1);
    BITFIELD_VALUE(write_through, 1);
    BITFIELD_VALUE(cache_disabled, 1);
    BITFIELD_VALUE(accessed, 1);
    BITFIELD_VALUE(_reserved, 1); // Must be 0
    BITFIELD_VALUE(large_page, 1);
    BITFIELD_VALUE(_avail1, 3); // Available for use by the OS
    BITFIELD_VALUE(pd_base_address, 40);
    BITFIELD_VALUE(_avail2, 11); // Available for use by the OS
    BITFIELD_VALUE(execute_disable, 1);
};

static_assert(
    sizeof(struct zerOS_pdp_entry) * 8 == 64,
    "struct zerOS_pdp_entry is not 64 bits wide"
);
static_assert(
    1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 3 + 40 + 11 + 1 == 64,
    "struct zerOS_pdp_entry bitfield values do not add up to 64"
);

struct TYPE_PACKED zerOS_pd_entry
{
    BITFIELD_VALUE(present, 1);
    BITFIELD_VALUE(read_write, 1);
    BITFIELD_VALUE(user_supervisor, 1);
    BITFIELD_VALUE(write_through, 1);
    BITFIELD_VALUE(cache_disabled, 1);
    BITFIELD_VALUE(accessed, 1);
    BITFIELD_VALUE(dirty, 1);
    BITFIELD_VALUE(_reserved, 1); // Must be 0
    BITFIELD_VALUE(global, 1);
    BITFIELD_VALUE(_avail1, 3); // Available for use by the OS
    BITFIELD_VALUE(pt_base_address, 40);
    BITFIELD_VALUE(_avail2, 11); // Available for use by the OS
    BITFIELD_VALUE(execute_disable, 1);
};

static_assert(
    sizeof(struct zerOS_pd_entry) * 8 == 64,
    "struct zerOS_pd_entry is not 64 bits wide"
);


#endif
