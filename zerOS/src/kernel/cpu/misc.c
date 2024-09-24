#include <kernel/cpu/misc.h>
#include <kernel/cpu/io.h>

#include <misc/sections.h>

extern void zerOS_halt(void)
{
    asm volatile(
        "hlt" : : : "memory"
    );
}

extern void zerOS_reboot(void)
{
    zerOS_outw(0x64, 0xFE | (1 << 8));
    zerOS_halt();
}

extern void zerOS_cli(void)
{
    asm volatile(
        "cli" : : : "memory"
    );
}

extern void zerOS_hcf(void)
{
    zerOS_cli();
    while (true)
    {
        zerOS_halt();
    }
}