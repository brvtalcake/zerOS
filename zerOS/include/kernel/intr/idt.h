#ifndef zerOS_KERNEL_INTR_IDT_H_INCLUDED
#define zerOS_KERNEL_INTR_IDT_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#include <misc/type.h>

#include <kernel/compiler/bitfield.h>

struct TYPE_PACKED idt_gate_descriptor_raw
{
    BITFIELD_VALUE(offset_low, 16);         // offset bits 0..15
    BITFIELD_VALUE(selector, 16);           // a code segment selector in GDT or LDT
    BITFIELD_VALUE(ist, 2);                 // bits 0..2 holds Interrupt Stack Table offset, rest of bits zero.
    BITFIELD_VALUE(reserved1, 6);           // remaining reserved bits for ist
    union TYPE_PACKED
    {
        BITFIELD_VALUE(type_attributes, 8); // gate type, dpl, and p fields
        struct TYPE_PACKED
        {
            BITFIELD_VALUE(gate_type, 4);
            BITFIELD_VALUE(unknown, 1);
            BITFIELD_VALUE(priv_lvl, 2);
            BITFIELD_VALUE(present, 1);
        };
    };
    BITFIELD_VALUE(offset_hi, 48);          // last offset bits
    BITFIELD_VALUE(reserved2, 32);          // reserved
};
typedef struct idt_gate_descriptor_raw idt_gate_desc_raw_t;

static_assert(
    sizeof(idt_gate_desc_raw_t) * 8 == 128,
    "64 bits IDT shall occupy 128 bits"
);

struct TYPE_PACKED idt_descriptor_raw
{
    uint16_t size;
    uint64_t offset;
};
typedef struct idt_descriptor_raw idt_desc_raw_t;

static_assert(
    sizeof(idt_desc_raw_t) * 8 == 80,
    "IDT descriptor shall be 80 bits"
);

#endif
