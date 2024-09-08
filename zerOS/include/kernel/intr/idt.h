#ifndef zerOS_KERNEL_INTR_IDT_H_INCLUDED
#define zerOS_KERNEL_INTR_IDT_H_INCLUDED

#include <stdint.h>
#include <stddef.h>
#include <assert.h>

#include <misc/type.h>

struct idt_descriptor_raw
{
    unsigned offset_low : 16;
} TYPE_PACKED;

#endif
