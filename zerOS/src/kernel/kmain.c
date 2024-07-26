#include <limine.h>
#include <stdint.h>
#include <stddef.h>

#include <boot/io.h>
#include <boot/cpu.h>
#include <boot/misc.h>
#include <boot/limine_setup.h>

void zerOS_kmain(void)
{
    // Fetch the first framebuffer.
    struct limine_framebuffer *framebuffer = get_framebuffers()->framebuffers[0];

    // Note: we assume the framebuffer model is RGB with 32-bit pixels.
    for (size_t i = 0; i < 100; i++) {
        volatile uint32_t *fb_ptr = framebuffer->address;
        fb_ptr[i * (framebuffer->pitch / 4) + i] = 0xffffff;
    }

    // We're done, just hang...
    while (true) zerOS_hcf();
}