#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <klibc/stdlib.h>
#include <klibc/string.h>
#include <limine.h>

LIMINE_BASE_REVISION(3);

uint8_t zerOS_inb(uint16_t port)
{
    uint8_t ret;
    asm volatile(
        "inb %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

uint16_t zerOS_inw(uint16_t port) {
    uint16_t ret;
    asm volatile(
        "inw %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

uint32_t zerOS_inl(uint16_t port)
{
    uint32_t ret;
    asm volatile(
        "inl %1, %0" :
        "=a"(ret)    :
        "Nd"(port)
    );
    return ret;
}

void zerOS_outb(uint16_t port, uint8_t val)
{
    asm volatile(
        "outb %0, %1" : : "a"(val), "Nd"(port)
    );
}

void zerOS_outw(uint16_t port, uint16_t val)
{
    asm volatile(
        "outw %0, %1" : : "a"(val), "Nd"(port)
    );
}

void zerOS_outl(uint16_t port, uint32_t val)
{
    asm volatile(
        "outl %0, %1" : : "a"(val), "Nd"(port)
    );
}

void zerOS_halt(void)
{
    asm volatile("hlt");
}

void zerOS_reboot(void)
{
    zerOS_outw(0x64, 0xFE | (1 << 8));
    zerOS_halt();
}

void zerOS_cli(void)
{
    asm volatile("cli");
}

void zerOS_hcf(void)
{
    zerOS_cli();
    while (true)
    {
        zerOS_halt();
    }
}