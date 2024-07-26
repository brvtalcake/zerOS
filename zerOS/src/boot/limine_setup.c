#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <klibc/stdlib.h>
#include <klibc/string.h>

#include <limine.h>

#include <misc/sections.h>
#include <misc/symbol.h>

#include <boot/io.h>
#include <boot/misc.h>

IN_SECTION(".requests") SYMBOL_USED
static volatile LIMINE_BASE_REVISION(2);

// The Limine requests can be placed anywhere, but it is important that
// the compiler does not optimise them away, so, usually, they should
// be made volatile or equivalent, _and_ they should be accessed at least
// once or marked as used with the "used" attribute as done here.

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = 0
};

// Finally, define the start and end markers for the Limine requests.
// These can also be moved anywhere, to any .c file, as seen fit.

IN_SECTION(".requests_start_marker") SYMBOL_USED
static volatile LIMINE_REQUESTS_START_MARKER;

IN_SECTION(".requests_end_marker") SYMBOL_USED
static volatile LIMINE_REQUESTS_END_MARKER;

extern struct limine_framebuffer_response* get_framebuffers(void)
{
    return framebuffer_request.response;
}

void zerOS_boot_setup(void)
{
    if (LIMINE_BASE_REVISION_SUPPORTED == false)
    {
        zerOS_hcf();
    }

    if (framebuffer_request.response == nullptr
     || framebuffer_request.response->framebuffer_count < 1) {
        zerOS_hcf();
    }

    void zerOS_kmain(void);
    zerOS_kmain();
}