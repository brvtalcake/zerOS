#include <limine.h>
#include <stdint.h>
#include <stddef.h>

#include <kernel/cpu/io.h>
#include <kernel/cpu/cpu.h>
#include <kernel/cpu/misc.h>

#include <kernel/limine_data.h>

void zerOS_kmain(void)
{
    struct limine_framebuffer* framebuffer = (struct limine_framebuffer*) zerOS_get_limine_data(zerOS_LIMINE_FRAMEBUFFER, (size_t)0);

    // Note: we assume the framebuffer model is RGB with 32-bit pixels.
    for (size_t i = 0; i < 100; i++) {
        volatile uint32_t* fb_ptr = framebuffer->address;
        fb_ptr[i * (framebuffer->pitch / 4) + i] = 0xffffff;
    }

    while (true) zerOS_hcf();
}