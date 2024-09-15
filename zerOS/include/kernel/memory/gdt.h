#ifndef zerOS_KERNEL_MEMORY_GDT_H_INCLUDED
#define zerOS_KERNEL_MEMORY_GDT_H_INCLUDED

#ifndef __ASSEMBLER__

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#include <misc/type.h>
#include <misc/bits.h>

#include <kernel/compiler/bitfield.h>
#include <kernel/compiler/enum.h>

#include <chaos/preprocessor/cat.h>

struct TYPE_PACKED zerOS_gdt_descriptor
{
    uint16_t size;
    uint64_t offset;
};

static_assert(
    sizeof(struct zerOS_gdt_descriptor) * 8 == 80,
    "GDT descriptor shall be 80 bits"
);

struct TYPE_PACKED zerOS_gdt_selector
{
    BITFIELD_VALUE(rpl, 2);
    BITFIELD_VALUE(table, 1);
    BITFIELD_VALUE(index, 13);
};

static_assert(
    sizeof(struct zerOS_gdt_selector) * 8 == 16,
    "GDT selector shall be 16 bits"
);

struct TYPE_PACKED zerOS_gdt_segment_registers
{
    struct zerOS_gdt_selector cs;
    struct zerOS_gdt_selector ds;
    struct zerOS_gdt_selector es;
    struct zerOS_gdt_selector fs;
    struct zerOS_gdt_selector gs;
    struct zerOS_gdt_selector ss;
};

static_assert(
    sizeof(struct zerOS_gdt_segment_registers) == sizeof(struct zerOS_gdt_selector) * 6,
    "Segment registers shall be 96 bits"
);

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
    BITFIELD_VALUE(_reserved, 1);
    BITFIELD_VALUE(long_mode, 1);
    BITFIELD_VALUE(size, 1);
    BITFIELD_VALUE(granularity, 1);
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
    BITFIELD_VALUE(available, 1);
    BITFIELD_VALUE(_unused, 2);
    BITFIELD_VALUE(granularity, 1);
    // END FLAGS

    BITFIELD_VALUE(base_hi, 40);
    BITFIELD_VALUE(_reserved, 32);
};

static_assert(
    sizeof(struct zerOS_gdt_system_segment_descriptor) == 2 * sizeof(struct zerOS_gdt_normal_segment_descriptor),
    "System GDT segment should occupy two times the space occupied by a normal one"
);

union TYPE_PACKED zerOS_gdt_entry
{
    struct zerOS_gdt_normal_segment_descriptor norm[2];
    struct zerOS_gdt_system_segment_descriptor sys;
};

static_assert(
    sizeof(union zerOS_gdt_entry) == sizeof(struct zerOS_gdt_system_segment_descriptor),
    "GDT entry should have the same size as a system segment descriptor"
);

typedef union zerOS_gdt_entry* zerOS_gdt_t;

#undef  zerOS_GDT_ENTRY_INDEX_NULL

#undef  zerOS_GDT_ENTRY_INDEX_KERNEL32_CS
#undef  zerOS_GDT_ENTRY_INDEX_KERNEL64_CS
#undef  zerOS_GDT_ENTRY_INDEX_KERNEL_DS

#undef  zerOS_GDT_ENTRY_INDEX_USER32_CS
#undef  zerOS_GDT_ENTRY_INDEX_USER_DS
#undef  zerOS_GDT_ENTRY_INDEX_USER64_CS

#undef  zerOS_GDT_ENTRY_INDEX_TSS

#undef  zerOS_GDT_ENTRY_INDEX_KERNEL_TLS
#undef  zerOS_GDT_ENTRY_INDEX_USER_TLS

#undef  zerOS_GDT_ENTRY_INDEX_MAX

/**
 * @def zerOS_GDT_ENTRY_INDEX_NULL
 * @brief The null segment index in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_NULL 0

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

/**
 * @def zerOS_GDT_ENTRY_INDEX_MAX
 * @brief The maximum number of entries in the GDT.
 */
#define zerOS_GDT_ENTRY_INDEX_MAX 16

#undef  zerOS_GDT_ENTRY_UNASSIGNED_INDEX
/**
 * @def zerOS_GDT_ENTRY_UNASSIGNED_INDEX
 * @brief Indexes of unassigned GDT entries.
 */
#define zerOS_GDT_ENTRY_UNASSIGNED_INDEX ((constexpr unsigned int[]){ 7, 12, 13, 14, 15 })

#undef  zerOS_GDT_ENTRY_INIT
/**
 * @def zerOS_GDT_ENTRY_INIT
 * @brief Initializer for a GDT entry.
 */
#define zerOS_GDT_ENTRY_INIT(kind) CHAOS_PP_CAT(__GDT_MK_ENTRY_, kind)

#undef  __GDT_NORMAL_ENTRY_BASE_LOW
#undef  __GDT_NORMAL_ENTRY_BASE_HIGH
#undef  __GDT_NORMAL_ENTRY_LIMIT_LOW
#undef  __GDT_NORMAL_ENTRY_LIMIT_HIGH

#define __GDT_NORMAL_ENTRY_BASE_LOW(base) ((base) & 0xFFFFFFU)
#define __GDT_NORMAL_ENTRY_BASE_HIGH(base) (((base) >> 24) & 0xFFU)
#define __GDT_NORMAL_ENTRY_LIMIT_LOW(limit) ((limit) & 0xFFFFU)
#define __GDT_NORMAL_ENTRY_LIMIT_HIGH(limit) (((limit) >> 16) & 0xFU)

#undef  __GDT_SYSTEM_ENTRY_BASE_LOW
#undef  __GDT_SYSTEM_ENTRY_BASE_HIGH
#undef  __GDT_SYSTEM_ENTRY_LIMIT_LOW
#undef  __GDT_SYSTEM_ENTRY_LIMIT_HIGH

#define __GDT_SYSTEM_ENTRY_BASE_LOW(base) __GDT_NORMAL_ENTRY_BASE_LOW(base)
#define __GDT_SYSTEM_ENTRY_BASE_HIGH(base) (((base) >> 24) & UINT64_C(0xFFFFFFFFFF))
#define __GDT_SYSTEM_ENTRY_LIMIT_LOW(limit) __GDT_NORMAL_ENTRY_LIMIT_LOW(limit)
#define __GDT_SYSTEM_ENTRY_LIMIT_HIGH(limit) __GDT_NORMAL_ENTRY_LIMIT_HIGH(limit)

#undef  __GDT_MK_ENTRY_null
#undef  __GDT_MK_ENTRY_normal
#undef  __GDT_MK_ENTRY_system

#define __GDT_MK_ENTRY_null(...) \
    ((struct zerOS_gdt_normal_segment_descriptor) { 0, 0, { 0, 0, 0, 0, 0, 0, 0 }, 0, 0, 0, 0, 0, 0, 0 })
#define __GDT_MK_ENTRY_normal(base, limit, access, flags)       \
    ((struct zerOS_gdt_normal_segment_descriptor) {             \
        .limit_low = __GDT_NORMAL_ENTRY_LIMIT_LOW(limit),       \
        .base_low = __GDT_NORMAL_ENTRY_BASE_LOW(base),          \
        .accessed = GET_BITS_AT(access, 0, 0),                  \
        .rw_bit = GET_BITS_AT(access, 1, 1),                    \
        .dc_bit = GET_BITS_AT(access, 2, 2),                    \
        .exec_bit = GET_BITS_AT(access, 3, 3),                  \
        .desc_type = 1,                                         \
        .priv_lvl = GET_BITS_AT(access, 5, 6),                  \
        .present = GET_BITS_AT(access, 7, 7),                   \
        .limit_hi = __GDT_NORMAL_ENTRY_LIMIT_HIGH(limit),       \
        ._reserved = 0,                                         \
        .long_mode = GET_BITS_AT(flags, 1, 1),                  \
        .size = GET_BITS_AT(flags, 2, 2),                       \
        .granularity = GET_BITS_AT(flags, 3, 3),                \
        .base_hi = __GDT_NORMAL_ENTRY_BASE_HIGH(base)           \
    })
#define __GDT_MK_ENTRY_system(base, limit, access, flags)       \
    ((struct zerOS_gdt_system_segment_descriptor) {             \
        .limit_low = __GDT_SYSTEM_ENTRY_LIMIT_LOW(limit),       \
        .base_low = __GDT_SYSTEM_ENTRY_BASE_LOW(base),          \
        .type = GET_BITS_AT(access, 0, 3),                      \
        .desc_type = 0,                                         \
        .priv_lvl = GET_BITS_AT(access, 5, 6),                  \
        .present = GET_BITS_AT(access, 7, 7),                   \
        .limit_hi = __GDT_SYSTEM_ENTRY_LIMIT_HIGH(limit),       \
        .available = GET_BITS_AT(flags, 0, 0),                  \
        ._unused = 0,                                           \
        .granularity = GET_BITS_AT(flags, 3, 3),                \
        .base_hi = __GDT_SYSTEM_ENTRY_BASE_HIGH(base),          \
        ._reserved = 0                                          \
    })

#undef  zerOS_MK_GDT_NORMAL_ENTRY_ACCESS
#undef  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS

/**
 * @def zerOS_MK_GDT_NORMAL_ENTRY_ACCESS
 * @brief Creates the access byte for a normal GDT entry.
 * @param accessed Is the segment accessed ?
 * @param rw_bit Additional read or write permissions
 * @param dc_bit Grows down or up ? (for data segments). Conforming ? (for code segments)
 * @param exec_bit Code segment if 1, data segment if 0
 * @param priv_lvl Privilege level
 * @param present Present bit
 * @return The access byte.
 * @see zerOS_GDT_ENTRY_INIT
 * @see zerOS_MK_GDT_NORMAL_ENTRY_FLAGS
 */
#define zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(   \
    accessed, rw_bit, dc_bit, exec_bit,     \
    priv_lvl, present                       \
)                                           \
    (                                       \
        ((accessed)  << 0) |                \
        ((rw_bit)    << 1) |                \
        ((dc_bit)    << 2) |                \
        ((exec_bit)  << 3) |                \
        ((1)         << 4) |                \
        ((priv_lvl)  << 5) |                \
        ((present)   << 7)                  \
    )
/**
 * @def zerOS_MK_GDT_NORMAL_ENTRY_FLAGS
 * @brief Creates the flags byte for a normal GDT entry.
 * @param long_mode Long mode bit
 * @param size Size bit (DB bit)
 * @param granularity Granularity bit
 * @return The flags byte.
 * @see zerOS_GDT_ENTRY_INIT
 * @see zerOS_MK_GDT_NORMAL_ENTRY_ACCESS
 */
#define zerOS_MK_GDT_NORMAL_ENTRY_FLAGS(    \
    long_mode, size, granularity            \
)                                           \
    (                                       \
        ((0)           << 0) |              \
        ((long_mode)   << 1) |              \
        ((size)        << 2) |              \
        ((granularity) << 3)                \
    )

#undef  zerOS_MK_GDT_SYSTEM_ENTRY_ACCESS
#undef  zerOS_MK_GDT_SYSTEM_ENTRY_FLAGS

/**
 * @def zerOS_MK_GDT_SYSTEM_ENTRY_ACCESS
 * @brief Creates the access byte for a system GDT entry.
 * @param type Type of system segment
 * @param priv_lvl Privilege level
 * @param present Present bit
 * @return The access byte.
 * @see zerOS_GDT_ENTRY_INIT
 * @see zerOS_MK_GDT_SYSTEM_ENTRY_FLAGS
 */
#define zerOS_MK_GDT_SYSTEM_ENTRY_ACCESS(   \
    type, priv_lvl, present                 \
)                                           \
    (                                       \
        ((type)      << 0) |                \
        ((0)         << 4) |                \
        ((priv_lvl)  << 5) |                \
        ((present)   << 7)                  \
    )
/**
 * @def zerOS_MK_GDT_SYSTEM_ENTRY_FLAGS
 * @brief Creates the flags byte for a system GDT entry.
 * @param long_mode Long mode bit
 * @param size Size bit (DB bit)
 * @param granularity Granularity bit
 * @return The flags byte.
 * @see zerOS_GDT_ENTRY_INIT
 * @see zerOS_MK_GDT_SYSTEM_ENTRY_ACCESS
 */
#define zerOS_MK_GDT_SYSTEM_ENTRY_FLAGS(    \
    available, granularity                  \
)                                           \
    (                                       \
        ((available)   << 0) |              \
        ((0)           << 1) |              \
        ((0)           << 2) |              \
        ((granularity) << 3)                \
    )

#undef  __KERNEL32_CS_ACCESS
#undef  __KERNEL32_CS_FLAGS
#define __KERNEL32_CS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 1, 0, 1)
#define __KERNEL32_CS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (0, 1, 1)

#undef  __KERNEL64_CS_ACCESS
#undef  __KERNEL64_CS_FLAGS
#define __KERNEL64_CS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 1, 0, 1)
#define __KERNEL64_CS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (1, 0, 1)

#undef  __KERNEL64_DS_ACCESS
#undef  __KERNEL64_DS_FLAGS
#define __KERNEL64_DS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 0, 0, 1)
#define __KERNEL64_DS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (0, 1, 1)

#undef  __USER32_CS_ACCESS
#undef  __USER32_CS_FLAGS
#define __USER32_CS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 1, 3, 1)
#define __USER32_CS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (0, 1, 1)

#undef  __USER64_CS_ACCESS
#undef  __USER64_CS_FLAGS
#define __USER64_CS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 1, 3, 1)
#define __USER64_CS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (1, 0, 1)

#undef  __USER64_DS_ACCESS
#undef  __USER64_DS_FLAGS
#define __USER64_DS_ACCESS zerOS_MK_GDT_NORMAL_ENTRY_ACCESS(1, 1, 0, 0, 3, 1)
#define __USER64_DS_FLAGS  zerOS_MK_GDT_NORMAL_ENTRY_FLAGS (0, 1, 1)

#undef  __TSS64_ACCESS
#undef  __TSS64_FLAGS
#define __TSS64_ACCESS zerOS_MK_GDT_SYSTEM_ENTRY_ACCESS(9, 0, 1)
#define __TSS64_FLAGS  zerOS_MK_GDT_SYSTEM_ENTRY_FLAGS (1, 1)

#undef  zerOS_GDT_ENTRY_NULL
/**
 * @def zerOS_GDT_ENTRY_NULL
 * @brief The null segment in the GDT.
 */
#define zerOS_GDT_ENTRY_NULL zerOS_GDT_ENTRY_INIT(null)()

#undef  zerOS_GDT_ENTRY_KERNEL32_CS
/**
 * @def zerOS_GDT_ENTRY_KERNEL32_CS
 * @brief The kernel's 32-bit code segment in the GDT.
 */
#define zerOS_GDT_ENTRY_KERNEL32_CS         \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __KERNEL32_CS_ACCESS,               \
        __KERNEL32_CS_FLAGS                 \
    )

#undef  zerOS_GDT_ENTRY_KERNEL64_CS
/**
 * @def zerOS_GDT_ENTRY_KERNEL64_CS
 * @brief The kernel's 64-bit code segment in the GDT.
 */
#define zerOS_GDT_ENTRY_KERNEL64_CS         \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __KERNEL64_CS_ACCESS,               \
        __KERNEL64_CS_FLAGS                 \
    )

#undef  zerOS_GDT_ENTRY_KERNEL_DS
/**
 * @def zerOS_GDT_ENTRY_KERNEL_DS
 * @brief The kernel's data segment in the GDT.
 */
#define zerOS_GDT_ENTRY_KERNEL_DS           \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __KERNEL64_DS_ACCESS,               \
        __KERNEL64_DS_FLAGS                 \
    )

#undef  zerOS_GDT_ENTRY_USER32_CS
/**
 * @def zerOS_GDT_ENTRY_USER32_CS
 * @brief The user's 32-bit code segment in the GDT.
 */
#define zerOS_GDT_ENTRY_USER32_CS           \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __USER32_CS_ACCESS,                 \
        __USER32_CS_FLAGS                   \
    )

#undef  zerOS_GDT_ENTRY_USER_DS
/**
 * @def zerOS_GDT_ENTRY_USER_DS
 * @brief The user's data segment in the GDT.
 */
#define zerOS_GDT_ENTRY_USER_DS             \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __USER64_DS_ACCESS,                 \
        __USER64_DS_FLAGS                   \
    )

#undef  zerOS_GDT_ENTRY_USER64_CS
/**
 * @def zerOS_GDT_ENTRY_USER64_CS
 * @brief The user's 64-bit code segment in the GDT.
 */
#define zerOS_GDT_ENTRY_USER64_CS           \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __USER64_CS_ACCESS,                 \
        __USER64_CS_FLAGS                   \
    )

#undef  zerOS_GDT_ENTRY_TSS
/**
 * @def zerOS_GDT_ENTRY_TSS
 * @brief The Task State Segment in the GDT.
 */
#define zerOS_GDT_ENTRY_TSS                 \
    zerOS_GDT_ENTRY_INIT(system)(           \
        0,                                  \
        0,                                  \
        __TSS64_ACCESS,                     \
        __TSS64_FLAGS                       \
    )

#undef  zerOS_GDT_ENTRY_KERNEL_TLS
/**
 * @def zerOS_GDT_ENTRY_KERNEL_TLS
 * @brief The kernel's Thread Local Storage segment in the GDT.
 */
#define zerOS_GDT_ENTRY_KERNEL_TLS          \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __KERNEL64_DS_ACCESS,               \
        __KERNEL64_DS_FLAGS                 \
    )

#undef  zerOS_GDT_ENTRY_USER_TLS
/**
 * @def zerOS_GDT_ENTRY_USER_TLS
 * @brief The user's Thread Local Storage segment in the GDT.
 */
#define zerOS_GDT_ENTRY_USER_TLS            \
    zerOS_GDT_ENTRY_INIT(normal)(           \
        0,                                  \
        UINT64_MAX,                         \
        __USER64_DS_ACCESS,                 \
        __USER64_DS_FLAGS                   \
    )

void zerOS_gdt_set(struct zerOS_gdt_descriptor* gdt_desc, struct zerOS_gdt_segment_registers* gdt_regs);

#else
.equ zerOS_GDT_ENTRY_INDEX_NULL, 0

.equ zerOS_GDT_ENTRY_INDEX_KERNEL32_CS, 1
.equ zerOS_GDT_ENTRY_INDEX_KERNEL64_CS, 2
.equ zerOS_GDT_ENTRY_INDEX_KERNEL_DS, 3

.equ zerOS_GDT_ENTRY_INDEX_USER32_CS, 4
.equ zerOS_GDT_ENTRY_INDEX_USER_DS, 5
.equ zerOS_GDT_ENTRY_INDEX_USER64_CS, 6

.equ zerOS_GDT_ENTRY_INDEX_TSS, 8

.equ zerOS_GDT_ENTRY_INDEX_KERNEL_TLS, 10
.equ zerOS_GDT_ENTRY_INDEX_USER_TLS, 11

.equ zerOS_GDT_ENTRY_INDEX_MAX, 16
#endif

#endif
