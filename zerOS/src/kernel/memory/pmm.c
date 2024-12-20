#include <config.h>

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <limine.h>

#include <klibc/alloca.h>
#include <klibc/assert.h>
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

// clang-format off
#undef  KLIBC_HARD_ASSERT_HOOK
#define KLIBC_HARD_ASSERT_HOOK(cond, file, line, func)  \
    do {                                                \
        zerOS_early_printk(                             \
            "zerOS: hard assertion failed: %s\n"        \
            "    at %s:%d in %s\n",                     \
            cond, file, line, func                      \
        );                                              \
        zerOS_hcf();                                    \
    } while (false)
// clang-format on

enum pmm_subregion_manager_tag
{
    PMM_SUBREGION_USABLE = 0,
    PMM_SUBREGION_BOOTLOADER_RECLAIMABLE,
    PMM_SUBREGION_FRAMEBUFFER,
    PMM_SUBREGION_KERNEL,
    PMM_SUBREGION_MODULES,
    PMM_SUBREGION_OTHER
};

struct pmm_subregion_manager_maybe_usable
{
    size_t limine_index;
    size_t next_free;
};

struct 

struct pmm_subregion_manager
{
    union
    {
        struct pmm_subregion_manager_maybe_usable usable; // Usable
        struct pmm_subregion_manager_maybe_usable boot;   // Bootloader reclaimable
        struct pmm_subregion_manager_framebuffer  fb;     // Framebuffer
        struct pmm_subregion_manager_kernel       kern;   // Kernel
        struct pmm_subregion_manager_modules      mod;    // Modules
        struct pmm_subregion_manager_other        other;  // Other
    };
    enum pmm_subregion_manager_tag tag;
    size_t size;
    size_t basepage_index;
};

/* struct pmm_basic_manager
{
    size_t   limine_entry_index; ///< The index of the limine entry.
    size_t   base_page_index;    ///< The index of the first page, in physical memory.
    bitset_t bitmap;             ///< The bitmap.
    bitset_t bitmap_physaddr;    ///< The physical address of the bitmap.
    size_t   next_free;          ///< The index of the next free page, in the bitmap.
    size_t   size;               ///< The size of the bitmap, in bits \
        (i.e. page_count or size_in_bytes * __CHAR_BIT__).
};

static struct pmm_basic_manager pmm_main_managers[zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS]; */
