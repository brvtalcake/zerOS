#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <klibc/stdlib.h>
#include <klibc/string.h>

#include <limine.h>

#include <misc/sections.h>
#include <misc/symbol.h>

#include <kernel/cpu/io.h>
#include <kernel/cpu/misc.h>
#include <kernel/memory/gdt.h>

#ifdef LIMINE_REQUESTED_REVISION
    #error "LIMINE_REQUESTED_REVISION shall not be defined before this point"
#endif
#define LIMINE_REQUESTED_REVISION (UINT64_C(2))

IN_SECTION(".requests") SYMBOL_USED
static LIMINE_BASE_REVISION(LIMINE_REQUESTED_REVISION);

// The Limine requests can be placed anywhere, but it is important that
// the compiler does not optimise them away, so, usually, they should
// be made volatile or equivalent, _and_ they should be accessed at least
// once or marked as used with the "used" attribute as done here.

// Ask Limine for 5LVL paging
IN_SECTION(".requests") SYMBOL_USED
static struct limine_paging_mode_request lvl5_paging_request = {
    .id = LIMINE_PAGING_MODE_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr,
    .mode = LIMINE_PAGING_MODE_X86_64_5LVL,
    .max_mode = LIMINE_PAGING_MODE_X86_64_5LVL,
    .min_mode = LIMINE_PAGING_MODE_X86_64_4LVL
};

IN_SECTION(".requests") SYMBOL_USED
static struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

// Finally, define the start and end markers for the Limine requests.
// These can also be moved anywhere, to any .c file, as seen fit.

IN_SECTION(".requests_start_marker") SYMBOL_USED
static LIMINE_REQUESTS_START_MARKER;

IN_SECTION(".requests_end_marker") SYMBOL_USED
static LIMINE_REQUESTS_END_MARKER;

extern struct limine_framebuffer_response* zerOS_get_limine_framebuffers(void)
{
    return framebuffer_request.response;
}

BOOT_FUNC
static inline void* boot_memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--)
        *d++ = *s++;
    return dest;
}

SYMBOL_ALIGNED(16)
static union zerOS_gdt_entry new_gdt_space[16];

BOOT_FUNC
static inline struct zerOS_gdt_descriptor* get_gdt_desc(void)
{
    SYMBOL_ALIGNED(16)
    static struct zerOS_gdt_descriptor gdt_desc;
    asm volatile(
        "sgdt %0"
        : "=m"(gdt_desc)
    );
    return &gdt_desc;
}

BOOT_FUNC
static inline void copy_limine_gdt(void)
{
    // Copy Limine's GDT into our GDT space
    struct zerOS_gdt_descriptor* limine_gdt = gdt_addr();
    boot_memcpy(new_gdt_space, (void*)limine_gdt->offset, limine_gdt->size);
}

BOOT_FUNC
static bool setup_gdt(void)
{
    // Already done by Limine
    // But replace it with our own GDT
}

BOOT_FUNC
static bool setup_paging(void)
{
}

BOOT_FUNC
static bool setup_idt(void)
{

}

BOOT_FUNC
static bool setup_isa_exts(void)
{
    
}

BOOT_FUNC
extern void zerOS_boot_setup(void)
{
    if (!setup_gdt())
        zerOS_hcf();

    if (!setup_paging())
        zerOS_hcf();

    if (!setup_idt())
        zerOS_hcf();

    if (!setup_isa_exts())
        zerOS_hcf();

    if (LIMINE_BASE_REVISION_SUPPORTED == false)
        zerOS_hcf();

    if (framebuffer_request.response == nullptr
     || framebuffer_request.response->framebuffer_count < 1) {
        zerOS_hcf();
    }

    void zerOS_kmain(void);
    zerOS_kmain();
}