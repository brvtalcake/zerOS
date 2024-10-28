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

struct pmm_basic_bitmap
{
    size_t managed_entries[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS];
    size_t managed_entry_count;
    bitset_t bitmap;
    bitset_t bitmap_physaddr;
    // ptr to the uint64_t where the next free page is
    size_t next_free;
    // size of the bitmap in bytes
    size_t size;
};

static struct pmm_basic_bitmap pmm_main_bitmap;

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

    memset(pmm_main_bitmap.managed_entries, 0, zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS * sizeof(size_t));
    pmm_main_bitmap.managed_entry_count = 0;

    for (size_t i = 0; i < memmap_entry_count && pmm_main_bitmap.managed_entry_count < zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
        if (entry->type == LIMINE_MEMMAP_USABLE || entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
        {
            pmm_main_bitmap.managed_entries[pmm_main_bitmap.managed_entry_count] = i;
            uint64_t div = entry->length / zerOS_PAGE_SIZE;
            uint64_t mod = entry->length % zerOS_PAGE_SIZE;
            if (unlikely(mod != 0))
            {
                zerOS_early_printk("zerOS: error: memmap entry length is not a multiple of the page size\n");
                zerOS_hcf();
            }
            count += div;
            ++pmm_main_bitmap.managed_entry_count;
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

    size_t bitmap_memmap_index, needed_pages;
    const uint64_t bitmap_phys_loc = get_bitmap_location(bitmap_size, &bitmap_memmap_index, &needed_pages);
    if (bitmap_phys_loc == (uint64_t)-1)
    {
        zerOS_early_printk("zerOS: error: failed to find a suitable location for the bitmap\n");
        zerOS_hcf();
    }
    zerOS_early_printk("zerOS: bitmap physical location: 0x%x\n", EPRI_CAST(x, bitmap_phys_loc));

    const uint64_t bitmap_virt_loc = hhdm_offset + bitmap_phys_loc;
    zerOS_early_printk("zerOS: bitmap virtual location: 0x%x\n", EPRI_CAST(x, bitmap_virt_loc));

    pmm_main_bitmap.bitmap          = (bitset_t)((uintptr_t)bitmap_virt_loc);
    pmm_main_bitmap.bitmap_physaddr = (bitset_t)((uintptr_t)bitmap_phys_loc);
    pmm_main_bitmap.size            = bitmap_size / sizeof(zerOS_fast_uint_t);
    pmm_main_bitmap.next_free       = 0;

    for (size_t i = 0; i < pmm_main_bitmap.managed_entry_count; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, pmm_main_bitmap.managed_entries[i]);
        if (entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
        {
            const size_t start = entry->base / zerOS_PAGE_SIZE;
            const size_t end   = (entry->base + entry->length) / zerOS_PAGE_SIZE;
            for (size_t j = start; j < end; j++)
                zerOS_bitset_set(pmm_main_bitmap.bitmap, j);
        }
        else if (entry->type == LIMINE_MEMMAP_USABLE)
        {
            const size_t start = entry->base / zerOS_PAGE_SIZE;
            const size_t end   = (entry->base + entry->length) / zerOS_PAGE_SIZE;
            for (size_t j = start; j < end; j++)
                zerOS_bitset_clear(pmm_main_bitmap.bitmap, j);
        }
    }

    bool managed_by(const struct pmm_basic_bitmap* bm, size_t ind)
    {
        for (size_t _i = 0; _i < bm->managed_entry_count; ++_i)
        {
            if (bm->managed_entries[_i] == ind)
                return true;
        }
        return false;
    };

    // Mark the bitmap itself as used
    size_t page_index = 0;
    for (size_t i = 0; i < bitmap_memmap_index; i++)
    {
        if (!managed_by(&pmm_main_bitmap, i))
            continue;
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
        for (size_t j = entry->base / zerOS_PAGE_SIZE; j < (entry->base + entry->length) / zerOS_PAGE_SIZE; j++)
        {
            if (j == bitmap_phys_loc / zerOS_PAGE_SIZE)
                zerOS_bitset_set(pmm_main_bitmap.bitmap, page_index);
            ++page_index;
        }
    }

    return true;
}

extern bool zerOS_pmm_alloc_frames(uintptr_t* filled, size_t count)
{
    return false;
}