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
    zerOS_PAGE_CONTAINERS_PRIVILEGE // Only when 5-level paging is enabled
};

#include <kernel/memory/paging_structures.h>

#endif
