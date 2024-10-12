#ifndef zerOS_KERNEL_MEMORY_PAGING_H_INCLUDED
#define zerOS_KERNEL_MEMORY_PAGING_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <misc/type.h>
#include <misc/statement.h>
#include <misc/unique_ident.h>

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

enum zerOS_page_table_type
{
    zerOS_PAGE_TABLE_PML4 = 0,
    zerOS_PAGE_TABLE_PDP,
    zerOS_PAGE_TABLE_PD,
    zerOS_PAGE_TABLE_PT
};

enum zerOS_page_translation_size
{
    zerOS_PAGE_TRANSLATION_4K = 0,
    zerOS_PAGE_TRANSLATION_2M,
    zerOS_PAGE_TRANSLATION_1G
};

enum zerOS_page_table_bits
    UNDERLYING_TYPE(uint64_t)
{
    zerOS_PAGE_PRESENT_BIT  = UINT64_C(1) << 0,
    zerOS_PAGE_RW_BIT       = UINT64_C(1) << 1,
    zerOS_PAGE_US_BIT       = UINT64_C(1) << 2,
    zerOS_PAGE_PWT_BIT      = UINT64_C(1) << 3,
    zerOS_PAGE_PCD_BIT      = UINT64_C(1) << 4,
    zerOS_PAGE_ACCESSED_BIT = UINT64_C(1) << 5,
    zerOS_PAGE_DIRTY_BIT    = UINT64_C(1) << 6,
    zerOS_PAGE_PS_BIT       = UINT64_C(1) << 7,
    zerOS_PAGE_GLOBAL_BIT   = UINT64_C(1) << 8,
    zerOS_PAGE_NX_BIT       = UINT64_C(1) << 63,

    zerOS_PAGE_PAT_PS0_BIT  = UINT64_C(1) << 7,  // When in Page Table Entry (i.e. PS bit is 0)
    zerOS_PAGE_PAT_PS1_BIT  = UINT64_C(1) << 12  // When in Page Directory Entry, or
                                                 // in Page Directory Pointer Entry (i.e. PS bit is 1)
};

/*
 * NOTE: `zerOS_PAGE_FLAG_<flag>` flags below are only an API, and not an actual representation
 * of a page table memory layout, hence, the values they are shifted to are not necessarily
 * meaningful.
 */

#undef  zerOS_PAGE_FLAG_PRESENT
#undef  zerOS_PAGE_FLAG_RW
#undef  zerOS_PAGE_FLAG_US
#undef  zerOS_PAGE_FLAG_PWT
#undef  zerOS_PAGE_FLAG_PCD
#undef  zerOS_PAGE_FLAG_ACCESSED
#undef  zerOS_PAGE_FLAG_DIRTY
#undef  zerOS_PAGE_FLAG_PS
#undef  zerOS_PAGE_FLAG_GLOBAL
#undef  zerOS_PAGE_FLAG_NX
#undef  zerOS_PAGE_FLAG_PKE

#undef  zerOS_PAGE_FLAG_CUSTOM

#define zerOS_PAGE_FLAG_PRESENT  (       UINT64_C( 1 ) << 0 )
#define zerOS_PAGE_FLAG_RW       (       UINT64_C( 1 ) << 1 )
#define zerOS_PAGE_FLAG_US       (       UINT64_C( 1 ) << 2 )
#define zerOS_PAGE_FLAG_PWT      (       UINT64_C( 1 ) << 3 )
#define zerOS_PAGE_FLAG_PCD      (       UINT64_C( 1 ) << 4 )
#define zerOS_PAGE_FLAG_ACCESSED (       UINT64_C( 1 ) << 5 )
#define zerOS_PAGE_FLAG_DIRTY    (       UINT64_C( 1 ) << 6 )
#define zerOS_PAGE_FLAG_PS       (       UINT64_C( 1 ) << 7 )
#define zerOS_PAGE_FLAG_GLOBAL   (       UINT64_C( 1 ) << 8 )
#define zerOS_PAGE_FLAG_NX       (       UINT64_C( 1 ) << 9 )
#define zerOS_PAGE_FLAG_PKE(val) (__PGFLAG_PKE_MK(val) << 10)

#define zerOS_PAGE_FLAG_CUSTOM(val) __PGFLAG_AVL_MK(val)

#undef  __PGFLAG_PKE_MK
#undef  __PGFLAG_PKE_GET
#undef  __PGFLAG_PKE_SHIFT
#define __PGFLAG_PKE_MK(v) ((uint64_t)v  & UINT64_C(0b1111))
#define __PGFLAG_PKE_GET(v) (((v) >> 10) & UINT64_C(0b1111))
#define __PGFLAG_PKE_SHIFT 59

#undef  __PGFLAG_AVL_MK
#undef  __PGFLAG_AVL_GET1
#undef  __PGFLAG_AVL_GET2
#undef  __PGFLAG_AVL_SHIFT1
#undef  __PGFLAG_AVL_SHIFT2
#define __PGFLAG_AVL_MK(v) ((uint64_t)v  & UINT64_C(0x3ff))
#define __PGFLAG_AVL_GET1(v) ( (v)       & UINT64_C(0b111))
#define __PGFLAG_AVL_GET2(v) (((v) >> 3) & UINT64_C(0x7f ))
#define __PGFLAG_AVL_SHIFT1 9
#define __PGFLAG_AVL_SHIFT2 52

#undef  zerOS_mk_page_struct
/**
 * @def zerOS_mk_page_struct(addr, tabletype, transsize, classicflags, customflags, pat)
 * @brief Fills a page structure (i.e. a table) accordingly.
 * @returns An `uint64_t` containing the new page table.
 * @note None of the flags in `classicflags` is checked.
 * @warning The provided address `addr` is not checked. If some bits "must-be-zero", then you'll need to do it yourself!
 */
#define zerOS_mk_page_struct(                                       \
    addr, tabletype, transsize, classicflags, customflags, pat      \
)                                                                   \
    ({                                                              \
        uint64_t UNIQUE(tbl) = 0;                                   \
        if ((classicflags) & zerOS_PAGE_FLAG_PRESENT)               \
            UNIQUE(tbl) |= zerOS_PAGE_PRESENT_BIT;                  \
        if ((classicflags) & zerOS_PAGE_FLAG_RW)                    \
            UNIQUE(tbl) |= zerOS_PAGE_RW_BIT;                       \
        if ((classicflags) & zerOS_PAGE_FLAG_US)                    \
            UNIQUE(tbl) |= zerOS_PAGE_US_BIT;                       \
        if ((classicflags) & zerOS_PAGE_FLAG_PWT)                   \
            UNIQUE(tbl) |= zerOS_PAGE_PWT_BIT;                      \
        if ((classicflags) & zerOS_PAGE_FLAG_PCD)                   \
            UNIQUE(tbl) |= zerOS_PAGE_PCD_BIT;                      \
        if ((classicflags) & zerOS_PAGE_FLAG_ACCESSED)              \
            UNIQUE(tbl) |= zerOS_PAGE_ACCESSED_BIT;                 \
        if ((classicflags) & zerOS_PAGE_FLAG_DIRTY)                 \
            UNIQUE(tbl) |= zerOS_PAGE_DIRTY_BIT;                    \
        if ((classicflags) & zerOS_PAGE_FLAG_PS)                    \
            UNIQUE(tbl) |= zerOS_PAGE_PS_BIT;                       \
        if ((classicflags) & zerOS_PAGE_FLAG_GLOBAL)                \
            UNIQUE(tbl) |= zerOS_PAGE_GLOBAL_BIT;                   \
        if ((classicflags) & zerOS_PAGE_FLAG_NX)                    \
            UNIQUE(tbl) |= zerOS_PAGE_NX_BIT;                       \
        if ((classicflags) & zerOS_PAGE_FLAG_PKE(0b1111))           \
            UNIQUE(tbl) |= __PGFLAG_PKE_GET((classicflags))         \
                        << __PGFLAG_PKE_SHIFT;                      \
                                                                    \
        if ((tabletype) == zerOS_PAGE_TABLE_PT)                     \
            UNIQUE(tbl) |= (pat) ? zerOS_PAGE_PAT_PS0_BIT : 0;      \
        else if ((tabletype) == zerOS_PAGE_TABLE_PD)                \
            UNIQUE(tbl) |= (pat) ? zerOS_PAGE_PAT_PS1_BIT : 0;      \
        else if ((tabletype) == zerOS_PAGE_TABLE_PDP)               \
            UNIQUE(tbl) |= (pat) ? zerOS_PAGE_PAT_PS1_BIT : 0;      \
                                                                    \
        if ((customflags))                                          \
        {                                                           \
            UNIQUE(tbl) |= __PGFLAG_AVL_GET1((customflags))         \
                        << __PGFLAG_AVL_SHIFT1;                     \
            UNIQUE(tbl) |= __PGFLAG_AVL_GET2((customflags))         \
                        << __PGFLAG_AVL_SHIFT2;                     \
        }                                                           \
                                                                    \
        bool UNIQUE(lowest) = false;                                \
        if ((tabletype) == zerOS_PAGE_TABLE_PT)                     \
            UNIQUE(lowest) = true;                                  \
        else if ((tabletype) == zerOS_PAGE_TABLE_PD &&              \
                 (transsize) == zerOS_PAGE_TRANSLATION_2M)          \
            UNIQUE(lowest) = true;                                  \
        else if ((tabletype) == zerOS_PAGE_TABLE_PDP &&             \
                 (transsize) == zerOS_PAGE_TRANSLATION_1G)          \
            UNIQUE(lowest) = true;                                  \
                                                                    \
        if (!UNIQUE(lowest))                                        \
            UNIQUE(tbl) |= (addr)                                   \
                        &  zerOS_pagetable_phyaddr_mask[0];         \
        else                                                        \
            UNIQUE(tbl) |= (addr)                                   \
                        &  zerOS_pagetable_phyaddr_mask[            \
                            (transsize)                             \
                        ];                                          \
                                                                    \
        UNIQUE(tbl);                                                \
    })

extern size_t zerOS_maxphyaddr;
extern uint64_t zerOS_pagetable_phyaddr_mask[3];

BOOT_FUNC
extern void zerOS_init_paging_values(void);

#endif
