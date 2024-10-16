#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <limine.h>

#include <misc/sections.h>

struct pmm_bitmap
{
    uint64_t* bitmap;
    size_t size;
};

BOOT_FUNC
extern bool zerOS_init_pmm(
    size_t** memory_ranges, size_t memory_ranges_count,
    struct limine_memmap_entry* entry_buf, size_t entry_count
)
{
    return true;
}