#include <boot/misc.h>
#include <boot/io.h>

#include <misc/sections.h>

BOOT_FUNC
extern void zerOS_halt(void)
{
    asm volatile("hlt");
}

BOOT_FUNC
extern void zerOS_reboot(void)
{
    zerOS_outw(0x64, 0xFE | (1 << 8));
    zerOS_halt();
}

BOOT_FUNC
extern void zerOS_cli(void)
{
    asm volatile("cli");
}

BOOT_FUNC
extern void zerOS_hcf(void)
{
    zerOS_cli();
    while (true)
    {
        zerOS_halt();
    }
}