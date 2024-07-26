#include <stddef.h>
#include <stdint.h>

#include <misc/sections.h>

BOOT_FUNC
extern uint8_t zerOS_inb(uint16_t port)
{
    uint8_t ret;
    asm volatile(
        "inb %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

BOOT_FUNC
extern uint16_t zerOS_inw(uint16_t port)
{
    uint16_t ret;
    asm volatile(
        "inw %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

BOOT_FUNC
extern uint32_t zerOS_inl(uint16_t port)
{
    uint32_t ret;
    asm volatile(
        "inl %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

BOOT_FUNC
extern void zerOS_outb(uint16_t port, uint8_t val)
{
    asm volatile(
        "outb %0, %1" : : "a"(val), "Nd"(port)
    );
}

BOOT_FUNC
extern void zerOS_outw(uint16_t port, uint16_t val)
{
    asm volatile(
        "outw %0, %1" : : "a"(val), "Nd"(port)
    );
}

BOOT_FUNC
extern void zerOS_outl(uint16_t port, uint32_t val)
{
    asm volatile(
        "outl %0, %1" : : "a"(val), "Nd"(port)
    );
}

BOOT_FUNC
extern uint64_t zerOS_read_msr(uint32_t msr)
{
    uint32_t lo, hi;
    asm volatile(
        "rdmsr" : "=a"(lo), "=d"(hi) : "c"(msr)
    );
    return ((uint64_t)hi << 32) | lo;
}

BOOT_FUNC
extern void zerOS_write_msr(uint32_t msr, uint64_t val)
{
    asm volatile(
        "wrmsr" : : "a"(val & 0xFFFFFFFF), "d"(val >> 32), "c"(msr)
    );
}