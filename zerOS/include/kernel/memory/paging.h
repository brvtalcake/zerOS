#ifndef zerOS_KERNEL_MEMORY_PAGING_H_INCLUDED
#define zerOS_KERNEL_MEMORY_PAGING_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <misc/type.h>

#include <kernel/compiler/bitfield.h>

#undef  zerOS_PAGE_SIZE
#define zerOS_PAGE_SIZE 4096ULL

enum zerOS_page_privilege_level
{
    zerOS_PAGE_KERNEL_PRIVILEGE = 0,
    zerOS_PAGE_DRIVER1_PRIVILEGE,
    zerOS_PAGE_DRIVER2_PRIVILEGE,
    zerOS_PAGE_USER_PRIVILEGE,
    zerOS_PAGE_CONTAINERS_PRIVILEGE
};

struct TYPE_PACKED zerOS_pml4_entry
{
    BITFIELD_VALUE(present, 1);
    BITFIELD_VALUE(rw, 1);
    BITFIELD_VALUE(us, 1);
    BITFIELD_VALUE(pwt, 1);
    BITFIELD_VALUE(pcd, 1);
    BITFIELD_VALUE(a, 1);
    BITFIELD_VALUE(_ignored, 1);
    BITFIELD_VALUE(_reserved1, 2);
    BITFIELD_VALUE(base, 40);
    BITFIELD_VALUE(_reserved2, 12);
};



#endif
