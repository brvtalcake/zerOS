#ifndef zerOS_KERNEL_MEMORY_GDT_H_INCLUDED
#define zerOS_KERNEL_MEMORY_GDT_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#include <misc/type.h>

#include <kernel/compiler/bitfield.h>

struct TYPE_PACKED gdt_normal_segment_descriptor_raw
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
#if 0
    union TYPE_PACKED
    {
        BITFIELD_VALUE(flags, 4);
        struct TYPE_PACKED
        {
            BITFIELD_VALUE(reserved, 1);
            BITFIELD_VALUE(granularity, 1);
            BITFIELD_VALUE(size, 1);
            BITFIELD_VALUE(long_mode, 1);
        };
    };
#else
    BITFIELD_VALUE(flags, 4);
#endif
    BITFIELD_VALUE(base_hi, 8);
};
typedef struct gdt_normal_segment_descriptor_raw gdt_norm_seg_desc_raw_t;

struct gdt_normal_segment_descriptor
{
    uint32_t base;
    _BitInt(20) limit;
    uint8_t access;
    uint8_t flags;
};

static_assert(
    sizeof(struct gdt_normal_segment_descriptor_raw) * 8 == 64,
    "Normal GDT segment should have a 64-bits size"
);

struct TYPE_PACKED gdt_system_segment_descriptor_raw
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
    BITFIELD_VALUE(flags, 4);
    BITFIELD_VALUE(base_hi, 40);
    BITFIELD_VALUE(reserved, 32);
};
typedef struct gdt_system_segment_descriptor_raw gdt_sys_seg_desc_raw_t;

static_assert(
    sizeof(struct gdt_system_segment_descriptor_raw) == 2 * sizeof(struct gdt_normal_segment_descriptor_raw),
    "System GDT segment should occupy two times the space occupied by a normal one"
);

struct TYPE_PACKED gdt_descriptor_raw
{
    uint16_t size;
    uint32_t offset;
};

struct gdt_descriptor
{
    uint32_t offset;
    uint16_t size;
};

#endif
