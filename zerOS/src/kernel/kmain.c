#include <stddef.h>
#include <stdint.h>

#include <limine.h>

#include <kernel/cpu/cpu.h>
#include <kernel/cpu/io.h>
#include <kernel/cpu/misc.h>
#include <kernel/limine_data.h>
#include <kernel/memory/paging.h>
#include <kernel/memory/pmm.h>
#include <kernel/memory/vmm.h>
#include <kernel/printk.h>
#include <kernel/qemu.h>
#include <kernel/serial/ports.h>

static bool setup_paging(void) { return true; }

static bool setup_printk_subsystem(void) { return true; }
static bool setup_idt(void) { return true; }

void zerOS_kmain(void)
{
    zerOS_early_printk("zerOS: setting up paging\n");
    if (!setup_paging())
    {
        zerOS_early_printk("zerOS: failed to setup paging\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: setting up IDT\n");
    if (!setup_idt())
    {
        zerOS_early_printk("zerOS: failed to setup IDT\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: setting up printk subsystem\n");
    if (!setup_printk_subsystem())
    {
        zerOS_early_printk("zerOS: failed to setup printk subsystem\n");
        zerOS_hcf();
    }

    struct limine_framebuffer* framebuffer =
      (struct limine_framebuffer*)zerOS_get_limine_data(zerOS_LIMINE_FRAMEBUFFER, (size_t)0);

    zerOS_early_printk("zerOS: writting to framebuffer to ensure everything is working\n");
    zerOS_early_printk(
      "zerOS: framebuffer virtual address: 0x%p\n", EPRI_CAST(p, framebuffer->address)
    );
    // zerOS_early_printk("zerOS: framebuffer physical address: 0x%p\n",
    //   zerOS_virt_to_phys((uintptr_t)framebuffer->address)
    // );
    //  Note: we assume the framebuffer model is RGB with 32-bit pixels.
    for (size_t i = 0; i < 100; i++)
    {
        volatile uint32_t* fb_ptr                = framebuffer->address;
        fb_ptr[i * (framebuffer->pitch / 4) + i] = 0xff'ff'ff;
    }

    while (true)
        zerOS_hcf();
}
