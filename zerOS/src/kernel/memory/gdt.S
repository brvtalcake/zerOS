# void zerOS_gdt_set(struct zerOS_gdt_descriptor* gdt_desc, struct zerOS_segment_registers* gdt_regs);
# Load the new GDT, and update the segment registers
#
# Inputs:
#   - %rdi: Pointer to the new GDT descriptor
#   - %rsi: Pointer to the new segment registers

# Here are the structures used in this function:
#
# struct zerOS_gdt_descriptor
# {
#     uint16_t size;
#     uint64_t offset;
# } __attribute__((packed));
#
# struct zerOS_gdt_selector
# {
#     unsigned rpl : 2;
#     unsigned table : 1;
#     unsigned index : 13;
# } __attribute__((packed));
# 
# struct zerOS_segment_registers
# {
#     struct zerOS_gdt_selector cs;
#     struct zerOS_gdt_selector ds;
#     struct zerOS_gdt_selector es;
#     struct zerOS_gdt_selector fs;
#     struct zerOS_gdt_selector gs;
#     struct zerOS_gdt_selector ss;
# } __attribute__((packed));

#undef  ASM_SYNTAX
#define ASM_SYNTAX ASM_SYNTAX_ATT

.include "asm/common.inc"

#include <asm/common.h>
#include <asm/syntax.h>

.global zerOS_gdt_set
.type zerOS_gdt_set, @function
.balign 16
zerOS_gdt_set:
    # Disable interrupts
    cli

    # Load the new GDT
    lgdt ACCESS_PTR(,%rdi,,,0)

    # Update the segment registers with the ones provided

    # Code segment register
    xor %rax, %rax
    mov16 ACCESS_PTR(,%rsi,,,0), %ax
    push64 %rax

    xor %rax, %rax
    leaq RIP_RELATIVE(.L__zerOS_on_new_cs), %rax
    push64 %rax
    lretq

.L__zerOS_on_new_cs:
    # Data segment registers
    mov16 ACCESS_PTR(,%rsi,,,2),  %ds
    mov16 ACCESS_PTR(,%rsi,,,4),  %es
    mov16 ACCESS_PTR(,%rsi,,,6),  %fs
    mov16 ACCESS_PTR(,%rsi,,,8),  %gs
    mov16 ACCESS_PTR(,%rsi,,,10), %ss

    # Enable interrupts
    sti

    # Return
    ret
