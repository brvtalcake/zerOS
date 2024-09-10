#ifndef zerOS_KERNEL_MEMORY_GDT_H_INCLUDED
#define zerOS_KERNEL_MEMORY_GDT_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#include <misc/type.h>

#include <kernel/compiler/bitfield.h>

struct TYPE_PACKED zerOS_gdt_normal_segment_descriptor
{
    BITFIELD_VALUE(limit_low, 16);
    BITFIELD_VALUE(base_low, 24);
    union TYPE_PACKED
    {
        BITFIELD_VALUE(access, 8);
        struct TYPE_PACKED
        {
            BITFIELD_VALUE(accessed, 1);  ///< Is the segment accessed ?
            BITFIELD_VALUE(rw_bit, 1);    ///< Additional read or write permissions
            BITFIELD_VALUE(dc_bit, 1);    ///< Grows down or up ? (for data segments). Conforming ? (for code segments)
            BITFIELD_VALUE(exec_bit, 1);  ///< Code segment if 1, data segment if 0
            BITFIELD_VALUE(desc_type, 1); ///< Descriptor type
            BITFIELD_VALUE(priv_lvl, 2);  ///< Privilege level
            BITFIELD_VALUE(present, 1);   ///< Present bit
        };
    };
    BITFIELD_VALUE(limit_hi, 4);

    // FLAGS
    BITFIELD_VALUE(reserved, 1);
    BITFIELD_VALUE(granularity, 1);
    BITFIELD_VALUE(size, 1);
    BITFIELD_VALUE(long_mode, 1);
    // END FLAGS

    BITFIELD_VALUE(base_hi, 8);
};

static_assert(
    sizeof(struct zerOS_gdt_normal_segment_descriptor) * 8 == 64,
    "Normal GDT segment should have a 64-bits size"
);

struct TYPE_PACKED zerOS_gdt_system_segment_descriptor
{
    BITFIELD_VALUE(limit_low, 16);
    BITFIELD_VALUE(base_low, 24);
    union TYPE_PACKED
    {
        BITFIELD_VALUE(access, 8);
        struct TYPE_PACKED
        {
            BITFIELD_VALUE(type, 4);      ///< Type of system segment
            BITFIELD_VALUE(desc_type, 1); ///< Descriptor type
            BITFIELD_VALUE(priv_lvl, 2);  ///< Privilege level
            BITFIELD_VALUE(present, 1);   ///< Present bit
        };
    };
    BITFIELD_VALUE(limit_hi, 4);

    // FLAGS
    BITFIELD_VALUE(reserved, 1);
    BITFIELD_VALUE(granularity, 1);
    BITFIELD_VALUE(size, 1);
    BITFIELD_VALUE(long_mode, 1);
    // END FLAGS

    BITFIELD_VALUE(base_hi, 40);
    BITFIELD_VALUE(reserved, 32);
};

static_assert(
    sizeof(struct zerOS_gdt_system_segment_descriptor) == 2 * sizeof(struct zerOS_gdt_normal_segment_descriptor),
    "System GDT segment should occupy two times the space occupied by a normal one"
);

struct TYPE_PACKED zerOS_gdt_descriptor
{
    uint16_t size;
    uint64_t offset;
};

static_assert(
    sizeof(struct zerOS_gdt_descriptor) * 8 == 80,
    "GDT descriptor shall be 80 bits"
);

union TYPE_PACKED zerOS_gdt_entry
{
    struct zerOS_gdt_normal_segment_descriptor norm[2];
    struct zerOS_gdt_system_segment_descriptor sys;
};

typedef union zerOS_gdt_entry* zerOS_gdt_t;

#undef  zerOS_GDT_ENTRY_INDEX_KERNEL32_CS
#undef  zerOS_GDT_ENTRY_INDEX_KERNEL64_CS
#undef  zerOS_GDT_ENTRY_INDEX_KERNEL_DS

#undef  zerOS_GDT_ENTRY_INDEX_USER32_CS
#undef  zerOS_GDT_ENTRY_INDEX_USER_DS
#undef  zerOS_GDT_ENTRY_INDEX_USER64_CS

#undef  zerOS_GDT_ENTRY_INDEX_TSS

#undef  zerOS_GDT_ENTRY_INDEX_KERNEL_TLS
#undef  zerOS_GDT_ENTRY_INDEX_USER_TLS

/**
 * @def zerOS_GDT_ENTRY_INDEX_KERNEL32_CS
 * @brief The kernel's 32-bit code segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_KERNEL32_CS 1
/**
 * @def zerOS_GDT_ENTRY_INDEX_KERNEL64_CS
 * @brief The kernel's 64-bit code segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_KERNEL64_CS 2
/**
 * @def zerOS_GDT_ENTRY_INDEX_KERNEL_DS
 * @brief The kernel's data segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_KERNEL_DS 3

/**
 * @def zerOS_GDT_ENTRY_INDEX_USER32_CS
 * @brief The user's 32-bit code segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_USER32_CS 4
/**
 * @def zerOS_GDT_ENTRY_INDEX_USER_DS
 * @brief The user's data segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_USER_DS 5
/**
 * @def zerOS_GDT_ENTRY_INDEX_USER64_CS
 * @brief The user's 64-bit code segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_USER64_CS 6

/**
 * @def zerOS_GDT_ENTRY_INDEX_TSS
 * @brief The Task State Segment index in the GDT.
 * @warning Needs 2 entries in the GDT as it is a "system segment".
 */
#define zerOS_GDT_ENTRY_INDEX_TSS 8

/**
 * @def zerOS_GDT_ENTRY_INDEX_KERNEL_TLS
 * @brief The kernel's Thread Local Storage segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_KERNEL_TLS 10
/**
 * @def zerOS_GDT_ENTRY_INDEX_USER_TLS
 * @brief The user's Thread Local Storage segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_USER_TLS 11



#endif
