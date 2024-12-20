#include <config.h>

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <limine.h>

#include <klibc/alloca.h>
#include <klibc/maybe.h>
#include <klibc/string.h>

#include <kernel/cpu/misc.h>
#include <kernel/data/bitset.h>
#include <kernel/limine_data.h>
#include <kernel/memory/align.h>
#include <kernel/memory/paging.h>
#include <kernel/printk.h>

#include <misc/sections.h>
#include <misc/symbol.h>
#include <misc/units.h>

#if 0
// TODO: Make this struct accept size in number of bits (i.e. number of pages) instead of size in number of zerOS_BITSET_UNDERLYING_TYPE elements
struct pmm_basic_manager
{
    size_t limine_entry_index; ///< The index of the limine entry.
    size_t base_page_index;    ///< The index of the first page, in physical memory.
    bitset_t bitmap;           ///< The bitmap.
    bitset_t bitmap_physaddr;  ///< The physical address of the bitmap.
    size_t next_free;          ///< The index of the next free page, in the bitmap.
    size_t size;               ///< The size of the bitmap, in bitmap elements (i.e. page_count / zerOS_fast_uint_bits or size_in_bytes / sizeof(zerOS_fast_uint_t)).
};
#else
struct pmm_basic_manager
{
    size_t   limine_entry_index; ///< The index of the limine entry.
    size_t   base_page_index;    ///< The index of the first page, in physical memory.
    bitset_t bitmap;             ///< The bitmap.
    bitset_t bitmap_physaddr;    ///< The physical address of the bitmap.
    size_t   next_free;          ///< The index of the next free page, in the bitmap.
    size_t   size;               ///< The size of the bitmap, in bits \
        (i.e. page_count or size_in_bytes * __CHAR_BIT__).
};
#endif

static struct pmm_basic_manager pmm_main_managers[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS];

static inline size_t calc_bitmap_size(size_t page_count)
{
    if (page_count % zerOS_fast_uint_bits != 0)
    {
        zerOS_early_printk(
          "zerOS: unhandled code path: page count "
          "is not a multiple of zerOS_fast_uint_bits\n"
        );
        zerOS_hcf();
    }
    const size_t bitmap_elem_count = page_count / zerOS_fast_uint_bits;
    return bitmap_elem_count * sizeof(zerOS_fast_uint_t);
}

// Returns the index of the memory map entry where the bitmap will be located
static size_t
find_proper_bitmaps_loc(size_t* manageable, size_t manageable_count, size_t memory_needs)
{
    for (size_t i = 0; i < manageable_count; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*)zerOS_get_limine_data(
          zerOS_LIMINE_MEMMAP_ENTRY, manageable[i]
        );
        const uint64_t entry_top = entry->base + entry->length;

        if (entry->type == LIMINE_MEMMAP_USABLE &&
            entry->length >= memory_needs &&
            entry_top <= UINT64_C(4) * GiB)
            return i;
    }

    // don't bother returning an error value
    // this shall not happen anyways, so just hang with an error message
    zerOS_early_printk("zerOS: error: failed to find a suitable location for the bitmaps\n");
    zerOS_hcf();
}

static void init_basic_managers(
  const uint64_t hhdm_offset, struct limine_memmap_entry* bitmap_entry, size_t* manageable,
  size_t* needs, size_t manageable_count, size_t needs_sum
)
{
    intptr_t current_offset = bitmap_entry->base + bitmap_entry->length;
    for (size_t i = manageable_count; i != 0; --i)
    {
        const size_t                realind = i - 1;
        struct limine_memmap_entry* entry   = (struct limine_memmap_entry*)zerOS_get_limine_data(
          zerOS_LIMINE_MEMMAP_ENTRY, manageable[realind]
        );

        current_offset -= (intptr_t)(needs[realind]);

        struct pmm_basic_manager* manager = &pmm_main_managers[realind];
        manager->limine_entry_index       = manageable[realind];
        manager->base_page_index          = entry->base / zerOS_PAGE_SIZE;
        manager->size                     = needs[realind] * 8;
        manager->bitmap_physaddr          = (bitset_t)current_offset;
        manager->bitmap                   = (bitset_t)(hhdm_offset + current_offset);
        manager->next_free                = 0;

        if (entry->type == LIMINE_MEMMAP_USABLE)
            zerOS_bitset_clear_all(manager->bitmap, manager->size);
        else if (entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
            zerOS_bitset_set_all(manager->bitmap, manager->size);

        if (entry == bitmap_entry)
        {
            const size_t max_managed_page_index = (entry->length / zerOS_PAGE_SIZE);
            if (max_managed_page_index != manager->size)
            {
                zerOS_early_printk("zerOS: logic error: assertion failed\n");
                zerOS_hcf();
            }

            const size_t occupied_pages =
              (needs_sum / zerOS_PAGE_SIZE) + ((needs_sum % zerOS_PAGE_SIZE) != 0 ? 1 : 0);
            for (size_t j = 0; j < occupied_pages; j++)
                zerOS_bitset_set(manager->bitmap, j);
        }
    }
}

static size_t __static_do_sum(size_t* arr, size_t count)
{
    size_t sum = 0;
    for (size_t i = 0; i < count; i++)
        sum += arr[i];
    return sum;
}

extern bool zerOS_init_pmm(void)
{
    struct limine_hhdm_response* hhdm =
      (struct limine_hhdm_response*)zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE);
    const uint64_t hhdm_offset = hhdm->offset;

    size_t manageable[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS];
    size_t manageable_count = 0;

    // in bytes
    size_t memory_needs[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS];

    struct limine_memmap_response* memmap_resp =
      (struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    const size_t memmap_entry_count = memmap_resp->entry_count;

    for (size_t i = 0; i < memmap_entry_count; i++)
    {
        struct limine_memmap_entry* entry =
          (struct limine_memmap_entry*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
        if (entry->type == LIMINE_MEMMAP_USABLE ||
            entry->type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
        {
            const uint64_t div = entry->length / zerOS_PAGE_SIZE,
                           mod = entry->length % zerOS_PAGE_SIZE;
            if (unlikely(mod != 0))
            {
                zerOS_early_printk(
                  "zerOS: error: memmap entry length is not a multiple of the page size\n"
                );
                zerOS_hcf();
            }

            size_t bmsize = calc_bitmap_size(div);
            // pmm_main_managers[manageable_count].limine_entry_index = i;
            // pmm_main_managers[manageable_count].base_page_index = entry->base / zerOS_PAGE_SIZE;
            // pmm_main_managers[manageable_count].size = bmsize / sizeof(zerOS_fast_uint_t);
            manageable[manageable_count]   = i;
            memory_needs[manageable_count] = bmsize;
            ++manageable_count;
        }
    }

    const size_t sum        = __static_do_sum(memory_needs, manageable_count);
    const size_t bitmap_loc = find_proper_bitmaps_loc(manageable, manageable_count, sum);
    struct limine_memmap_entry* bitmap_entry =
      (struct limine_memmap_entry*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, bitmap_loc);

    init_basic_managers(hhdm_offset, bitmap_entry, manageable, memory_needs, manageable_count, sum);
}

extern bool zerOS_pmm_alloc_frames(uintptr_t* filled, size_t count)
{
    // TODO
    return false;
}
