#include <config.h>

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <limine.h>

#include <kernel/printk.h>
#include <kernel/limine_data.h>
#include <kernel/memory/paging.h>
#include <kernel/data/bitset.h>
#include <kernel/cpu/misc.h>

#include <misc/sections.h>
#include <misc/symbol.h>
#include <misc/units.h>

#include <klibc/maybe.h>
#include <klibc/alloca.h>
#include <klibc/string.h>

struct pmm_basic_manager
{
    size_t limine_entry_index; ///< The index of the limine entry.
    size_t base_page_index;    ///< The index of the first page, in physical memory.
    bitset_t bitmap;           ///< The bitmap.
    bitset_t bitmap_physaddr;  ///< The physical address of the bitmap.
    size_t next_free;          ///< The index of the next free page, in the bitmap.
    size_t size;               ///< The size of the bitmap, in pages.
};

static struct pmm_basic_manager pmm_main_managers[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS];

static uintptr_t get_cr3(void)
{
    uintptr_t ret;
    asm volatile(
        "mov %%cr3, %0" : "=r"(ret)
    );
    return ret;
}

/**
 * @brief Gives a location where the bitmap shall reside.
 * @param bitmap_size The size of the bitmap in bytes.
 * @param memmap_index_out The index of the memory map entry where the bitmap is located.
 * @return A proper (physical) address for the start of the bitmap.
 * @warning This function is only to be used in the early paging setup.
 */
static uint64_t get_bitmap_location(size_t bitmap_size, size_t* memmap_index_out, size_t* needed_pages)
{
    struct limine_memmap_response* memmap_resp = (struct limine_memmap_response*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    const size_t count = memmap_resp->entry_count;
    
    bool found = false;
    uint64_t entry_top, entry_base;
    
    *needed_pages = (bitmap_size / zerOS_PAGE_SIZE) + 1;
    zerOS_early_printk("zerOS: needed pages: %u\n", EPRI_CAST(u, *needed_pages));
    
    for (size_t where = count; where != 0 && !found; --where)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, where - 1);
        entry_base = entry->base;
        entry_top  = entry_base + entry->length;

        if (entry->type   == LIMINE_MEMMAP_USABLE &&
            entry->length >= bitmap_size        &&
            entry_top     <= UINT64_C(4) * GiB)
        {
            *memmap_index_out = where - 1;
            found = true;
        }
    }

    if (!found)
        return (uint64_t)-1;

    return entry_top - bitmap_size;
}

// Simply count the number of free usable and bootloader reclaimable pages
static size_t get_managed_mem_size(void)
{
    struct limine_memmap_response* memmap_resp = (struct limine_memmap_response*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    struct limine_memmap_entry* memmap = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, (size_t)0);
    const size_t memmap_entry_count = memmap_resp->entry_count;

    size_t count = 0;

    memset(pmm_main_managers, 0, zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS * sizeof(struct pmm_basic_manager));

    size_t manager_index = 0;
    for (size_t i = 0; i < memmap_entry_count; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
        if (entry->type == LIMINE_MEMMAP_USABLE || entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
        {
            uint64_t div = entry->length / zerOS_PAGE_SIZE;
            uint64_t mod = entry->length % zerOS_PAGE_SIZE;
            if (unlikely(mod != 0))
            {
                zerOS_early_printk("zerOS: error: memmap entry length is not a multiple of the page size\n");
                zerOS_hcf();
            }
            count += div;
            pmm_main_managers[manager_index].limine_entry_index = i;
            pmm_main_managers[manager_index].base_page_index = entry->base / zerOS_PAGE_SIZE;
        }
    }

    return count;
}

extern bool zerOS_init_pmm(void)
{
    struct limine_hhdm_response* hhdm = (struct limine_hhdm_response*) zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE);
    const uint64_t hhdm_offset = hhdm->offset;
    
    // In number of pages
    const size_t physmem  = get_managed_mem_size();
    zerOS_early_printk("zerOS: found %u pages of usable memory\n", EPRI_CAST(u, physmem));
    
    // zerOS_fast_uint_bits pages per zerOS_fast_uint_t
    const size_t in_bytes = (physmem / zerOS_fast_uint_bits) + 1;
    zerOS_early_printk("zerOS: bitmap size: %u bytes\n", EPRI_CAST(u, in_bytes));
    
    const size_t bitmap_size = in_bytes * sizeof(zerOS_fast_uint_t);
    zerOS_early_printk("zerOS: bitmap size: %u bytes\n", EPRI_CAST(u, bitmap_size));

    size_t bitmap_memmap_index, bitmap_pages;
    const uint64_t bitmap_phys_loc = get_bitmap_location(bitmap_size, &bitmap_memmap_index, &bitmap_pages);
    if (bitmap_phys_loc == (uint64_t)-1)
    {
        zerOS_early_printk("zerOS: error: failed to find a suitable location for the bitmap\n");
        zerOS_hcf();
    }
    zerOS_early_printk("zerOS: bitmap physical location: 0x%x\n", EPRI_CAST(x, bitmap_phys_loc));

    const uint64_t bitmap_virt_loc = hhdm_offset + bitmap_phys_loc;
    zerOS_early_printk("zerOS: bitmap virtual location: 0x%x\n", EPRI_CAST(x, bitmap_virt_loc));

    pmm_main_managers.bitmap          = (bitset_t)((uintptr_t)bitmap_virt_loc);
    pmm_main_managers.bitmap_physaddr = (bitset_t)((uintptr_t)bitmap_phys_loc);
    pmm_main_managers.size            = bitmap_size / sizeof(zerOS_fast_uint_t);
    pmm_main_managers.next_free       = 0;

    bool managed_by(const struct pmm_basic_manager* bm, size_t ind)
    {
        for (size_t _i = 0; _i < bm->managed_entry_count; ++_i)
        {
            if (bm->managed_entries[_i] == ind)
                return true;
        }
        return false;
    };

    // By default, set all `LIMINE_MEMMAP_USABLE` pages as free,
    // and all `LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE` pages as used
    // Also mark the bitmap itself as used
    size_t page_index = 0;
    for (size_t i = 0; i < pmm_main_managers.managed_entry_count; i++)
    {
        struct limine_memmap_entry* entry = managed_entry_get_limine_entry(&pmm_main_managers, i);
        if (entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
        {
            const size_t end_cond = page_index + (entry->length / zerOS_PAGE_SIZE);
            while (page_index < end_cond)
            {
                zerOS_bitset_set(pmm_main_managers.bitmap, page_index);
                ++page_index;
            }
        }
        else if (entry->type == LIMINE_MEMMAP_USABLE)
        {
        }
    }
    
    return true;
}

extern bool zerOS_pmm_alloc_frames(uintptr_t* filled, size_t count)
{
    // TODO
    return false;
}