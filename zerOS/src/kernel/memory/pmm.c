#include <config.h>

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <limine.h>

#include <kernel/limine_data.h>
#include <kernel/memory/paging.h>
#include <kernel/data/bitset.h>

#include <misc/sections.h>
#include <misc/symbol.h>

#include <klibc/maybe.h>
#include <klibc/alloca.h>

#undef  GiB
#define GiB (UINT64_C(1024) * UINT64_C(1024) * UINT64_C(1024))

struct pmm_bitmap
{
    uintptr_t physaddr_start;
    uintptr_t physaddr_end;
    // bitmap pointer itself
    bitset_t bitmap;
    // ptr to the uint64_t where the next free page is
    size_t next_free;
    // size of the bitmap in bytes
    size_t size;
};

static struct pmm_bitmap pmm_bitmap;

static inline bool pmm_is_page_free(size_t page)
{
    return !zerOS_bitset_test(pmm_bitmap.bitmap, page);
}

static inline void pmm_set_page(size_t page)
{
    zerOS_bitset_set(pmm_bitmap.bitmap, page);
}

static inline void pmm_clear_page(size_t page)
{
    zerOS_bitset_clear(pmm_bitmap.bitmap, page);
}

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
 * @return A proper (physical) address for the start of the bitmap.
 * @warning This function is only to be used in the early paging setup.
 */
static uint64_t get_bitmap_location(size_t bitmap_size)
{
    struct limine_memmap_response* memmap_resp = (struct limine_memmap_response*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    const size_t count = memmap_resp->entry_count;
    
    bool found = false;
    size_t where;
    uint64_t entry_top, entry_base;
    
    for (where = count; where != 0 && !found; --where)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, where - 1);
        entry_base = entry->base;
        entry_top  = entry_base + entry->length;

        if (entry->type   == LIMINE_MEMMAP_USABLE &&
            entry->length >= bitmap_size        &&
            entry_top     <= UINT64_C(4) * GiB)
            found = true;
    }

    if (!found)
        return (uint64_t)-1;

    return entry_top - bitmap_size;
}

// Simply count the number of free usable and bootloader reclaimable pages
static size_t get_physmem_size(void)
{
    struct limine_memmap_response* memmap_resp = (struct limine_memmap_response*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    struct limine_memmap_entry* memmap = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, (size_t)0);
    const size_t memmap_entry_count = memmap_resp->entry_count;

    size_t count = 0;

    for (size_t i = 0; i < memmap_entry_count; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*) zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
        if (entry->type == LIMINE_MEMMAP_USABLE || entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
            count += entry->length / zerOS_PAGE_SIZE;
    }

    return count;
}

extern bool zerOS_init_pmm(void)
{
    struct limine_hhdm_response* hhdm = (struct limine_hhdm_response*) zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE);
    const uint64_t hhdm_offset = hhdm->offset;
    
    // In number of pages
    const size_t physmem = get_physmem_size();
    // In bytes
    const size_t bitmap_size = (physmem / 8) + 1;
    const uintptr_t bitmap_physaddr = get_bitmap_location(bitmap_size);
    
    if (bitmap_physaddr == (size_t)-1)
        return false;

    const size_t bitmap_virtaddr = hhdm_offset + bitmap_physaddr;

    pmm_bitmap.physaddr_start = bitmap_physaddr;
    pmm_bitmap.physaddr_end   = bitmap_physaddr + bitmap_size;
    pmm_bitmap.bitmap         = (bitset_t)bitmap_virtaddr;
    pmm_bitmap.next_free      = 0;
    pmm_bitmap.size           = physmem;
    zerOS_bitset_set_all(pmm_bitmap.bitmap, physmem);

    return true;
}

extern bool zerOS_pmm_alloc_frames(uintptr_t* filled, size_t count)
{
    if (count == 0)
        return nullptr;

    
    const size_t i    = pmm_bitmap.next_free;
    const size_t size = pmm_bitmap.size;
    
    size_t found = 0;
    size_t j;

    for (j = 0; j < size && found < count; ++j)
    {
        const size_t index = (i + j) % size;
    
        if (pmm_is_page_free(index))
        {
            pmm_set_page(index);
            filled[found++] = pmm_bitmap.physaddr_start + index * 4096;
        }
    }

    if (found == count)
    {
        pmm_bitmap.next_free = (i + j) % size;
        return true;
    }

    for (size_t k = 0; k < found; ++k)
        pmm_clear_page((filled[k] - pmm_bitmap.physaddr_start) / 4096);

    return false;
}