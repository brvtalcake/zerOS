#include <config.h>

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <klibc/stdlib.h>
#include <klibc/string.h>

#include <limine.h>

#include <misc/sections.h>
#include <misc/symbol.h>
#include <misc/array.h>

#include <kernel/printk.h>
#include <kernel/cpu/io.h>
#include <kernel/cpu/misc.h>
#include <kernel/memory/gdt.h>
#include <kernel/memory/paging.h>
#include <kernel/serial/ports.h>

#include <machine/setup.h>

#include <chaos/preprocessor/cat.h>

#undef  calloca
#define calloca(size)                               \
    ({                                              \
        char* UNIQUE(ptr) = __builtin_alloca(size)  \
        for (                                       \
            size_t UNIQUE(i) = 0;                   \
            UNIQUE(i) < size;                       \
            UNIQUE(i)++                             \
        ) UNIQUE(ptr)[UNIQUE(i)] = 0;               \
        UNIQUE(ptr);                                \
    })

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

IN_SECTION(".requests") SYMBOL_USED
static struct limine_firmware_type_request firmware_type_request = {
    .id = LIMINE_FIRMWARE_TYPE_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static struct limine_hhdm_request hhdm_request = {
    .id = LIMINE_HHDM_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static struct limine_efi_memmap_request efi_memmap_request = {
    .id = LIMINE_EFI_MEMMAP_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static struct limine_efi_system_table_request efi_system_table_request = {
    .id = LIMINE_EFI_SYSTEM_TABLE_REQUEST,
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

SYMBOL_ALIGNED_TO(zerOS_PAGE_SIZE) SYMBOL_USED
static unsigned char new_gdt_space[zerOS_GDT_ENTRY_INDEX_MAX * sizeof(struct zerOS_gdt_normal_segment_descriptor)];

BOOT_FUNC
static bool setup_kern_modules(void)
{ return true; }

BOOT_FUNC
static size_t get_needed_mem_entries(struct limine_memmap_entry* entry_buf)
{
    struct limine_memmap_response* response = memmap_request.response;
    if (!response)
        return 0;
    size_t count = 0;
    for (size_t i = 0; i < response->entry_count; i++)
    {
        struct limine_memmap_entry* entry = response->entries[i];
        switch (entry->type)
        {
            case LIMINE_MEMMAP_USABLE:
                CASE_FALLTHROUGH;
            case LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE:
                boot_memcpy(&entry_buf[count++], entry, sizeof(struct limine_memmap_entry));
                break;
            default: continue;
        }
    }
    return count;
}

BOOT_FUNC
static bool assert_uefi_x86_64(void)
{
    struct limine_firmware_type_response* response = firmware_type_request.response;
    if (!response)
        return false;
    return response->firmware_type == LIMINE_FIRMWARE_TYPE_UEFI64;
}

BOOT_FUNC
static bool fill_unassigned_gdtent(void)
{
    const unsigned int unassigned[] = zerOS_GDT_ENTRY_UNASSIGNED_INDEX;
    SYMBOL_ALIGNED_TO(16)
    const struct zerOS_gdt_normal_segment_descriptor unassigned_desc = zerOS_GDT_ENTRY_NULL;
    for (size_t i = 0; i < ARRAY_LEN(unassigned); i++)
    {
        const unsigned int idx = unassigned[i];
        const unsigned int real_idx = idx & (~1U);
        if (idx == real_idx)
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[0]);
            boot_memcpy(desc, &unassigned_desc, sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
        else
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[1]);
            boot_memcpy(desc, &unassigned_desc, sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
    }

    return true;
}

BOOT_FUNC
static bool fill_null_gdtent(void)
{
    SYMBOL_ALIGNED_TO(16)
    const struct zerOS_gdt_normal_segment_descriptor null_desc = zerOS_GDT_ENTRY_NULL;
    boot_memcpy(new_gdt_space, &null_desc, sizeof(struct zerOS_gdt_normal_segment_descriptor));
    return fill_unassigned_gdtent();
}

BOOT_FUNC
static bool setup_normsegs(void)
{
    const unsigned int normsegs[] = {
        zerOS_GDT_ENTRY_INDEX_NULL,

        zerOS_GDT_ENTRY_INDEX_KERNEL32_CS,
        zerOS_GDT_ENTRY_INDEX_KERNEL64_CS,
        zerOS_GDT_ENTRY_INDEX_KERNEL_DS,

        zerOS_GDT_ENTRY_INDEX_USER32_CS,
        zerOS_GDT_ENTRY_INDEX_USER64_CS,
        zerOS_GDT_ENTRY_INDEX_USER_DS
    };

    SYMBOL_ALIGNED_TO(16)
    const struct zerOS_gdt_normal_segment_descriptor normsegs_desc[] = {
        zerOS_GDT_ENTRY_NULL,

        zerOS_GDT_ENTRY_KERNEL32_CS,
        zerOS_GDT_ENTRY_KERNEL64_CS,
        zerOS_GDT_ENTRY_KERNEL_DS,

        zerOS_GDT_ENTRY_USER32_CS,
        zerOS_GDT_ENTRY_USER64_CS,
        zerOS_GDT_ENTRY_USER_DS
    };

    for (size_t i = 0; i < ARRAY_LEN(normsegs); i++)
    {
        const unsigned int idx = normsegs[i];
        const unsigned int real_idx = idx & (~1U);
        if (idx == real_idx)
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[0]);
            boot_memcpy(desc, &normsegs_desc[i], sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
        else
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[1]);
            boot_memcpy(desc, &normsegs_desc[i], sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
    }

    return true;
}

BOOT_FUNC
static bool setup_syssegs(void)
{
    const unsigned int syssegs[] = {
        zerOS_GDT_ENTRY_INDEX_TSS
    };

    SYMBOL_ALIGNED_TO(16)
    const struct zerOS_gdt_system_segment_descriptor syssegs_desc[] = {
        zerOS_GDT_ENTRY_TSS
    };

    for (size_t i = 0; i < ARRAY_LEN(syssegs); i++)
    {
        const unsigned int idx = syssegs[i];
        const unsigned int real_idx = idx & (~1U);
        if (real_idx != idx)
            return false;
        const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
        union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
        struct zerOS_gdt_system_segment_descriptor* desc = &(entry->sys);
        boot_memcpy(desc, &syssegs_desc[i], sizeof(struct zerOS_gdt_system_segment_descriptor));
    }

    return true;
}

BOOT_FUNC
static bool setup_tlssegs(void)
{
    const unsigned int tlssegs[] = {
        zerOS_GDT_ENTRY_INDEX_KERNEL_TLS,
        zerOS_GDT_ENTRY_INDEX_USER_TLS
    };

    SYMBOL_ALIGNED_TO(16)
    const struct zerOS_gdt_normal_segment_descriptor tlssegs_desc[] = {
        zerOS_GDT_ENTRY_KERNEL_TLS,
        zerOS_GDT_ENTRY_USER_TLS
    };

    for (size_t i = 0; i < ARRAY_LEN(tlssegs); i++)
    {
        const unsigned int idx = tlssegs[i];
        const unsigned int real_idx = idx & (~1U);
        if (idx == real_idx)
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[0]);
            boot_memcpy(desc, &tlssegs_desc[i], sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
        else
        {
            const size_t offset = real_idx * sizeof(struct zerOS_gdt_normal_segment_descriptor);
            union zerOS_gdt_entry* entry = (union zerOS_gdt_entry*)(new_gdt_space + offset);
            struct zerOS_gdt_normal_segment_descriptor* desc = &(entry->norm[1]);
            boot_memcpy(desc, &tlssegs_desc[i], sizeof(struct zerOS_gdt_normal_segment_descriptor));
        }
    }

    return true;
}

BOOT_FUNC
static bool load_new_gdt(void)
{
    SYMBOL_ALIGNED_TO(16)
    struct zerOS_gdt_descriptor gdt_desc;
    SYMBOL_ALIGNED_TO(16)
    struct zerOS_gdt_segment_registers gdt_regs;

    gdt_desc.offset = (uint64_t)(void*)new_gdt_space;
    gdt_desc.size   = (zerOS_GDT_ENTRY_INDEX_MAX * sizeof(struct zerOS_gdt_normal_segment_descriptor)) - 1;

    gdt_regs.cs = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL64_CS,
        .rpl = 0,
        .table = 0
    };
    gdt_regs.ds = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL_DS,
        .rpl = 0,
        .table = 0
    };
    gdt_regs.es = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL_DS,
        .rpl = 0,
        .table = 0
    };
    gdt_regs.fs = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL_DS,
        .rpl = 0,
        .table = 0
    };
    gdt_regs.gs = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL_DS,
        .rpl = 0,
        .table = 0
    };
    gdt_regs.ss = (struct zerOS_gdt_selector){
        .index = zerOS_GDT_ENTRY_INDEX_KERNEL_DS,
        .rpl = 0,
        .table = 0
    };

    zerOS_gdt_set(&gdt_desc, &gdt_regs);

    return true;
}

BOOT_FUNC
static bool setup_gdt(void)
{
    // Already set up by Limine
    // But replace it with our own GDT

    bool ret = true;
    zerOS_early_printk("zerOS: filling up new GDT\n");
    ret = ret && setup_normsegs();
    ret = ret && setup_syssegs();
    ret = ret && setup_tlssegs();
    ret = ret && fill_null_gdtent();
    zerOS_early_printk("zerOS: loading new GDT\n");
    ret = ret && load_new_gdt();
    return ret;
}

BOOT_FUNC
static bool setup_paging(void)
{
    zerOS_init_paging_values();

    if (lvl5_paging_request.response == nullptr)
        return false;

    if (
        lvl5_paging_request.response->mode != LIMINE_PAGING_MODE_X86_64_5LVL &&
        lvl5_paging_request.response->mode != LIMINE_PAGING_MODE_X86_64_4LVL
    )
        return false;

    struct limine_memmap_entry* entry_buf = calloca(sizeof(struct limine_memmap_entry) * memmap_request.response->entry_count);
    const size_t entry_count = get_needed_mem_entries(entry_buf);

    if (entry_count == 0)
        return false;

    if (!zerOS_init_pmm(entry_buf, entry_count))
        return false;

    if (!zerOS_init_vmm())
        return false;

    return true;
}

BOOT_FUNC
static bool setup_idt(void)
{
    return true;
}

/**
 * @brief Setup ISA extensions (such as SSE, AVX, etc.) that GCC might use
 * when generating code.
 */
BOOT_FUNC
static bool setup_isa_exts(void)
{
    return CHAOS_PP_CAT(zerOS_CONFIG_CPU, _setup_isa_exts) ();
}

BOOT_FUNC
static bool setup_early_debug(void)
{
    if (!zerOS_serial_early_init())
        return false;

    return true;
}

BOOT_FUNC
extern void zerOS_boot_setup(void)
{
    /* if (!zerOS_stage_set(zerOS_STAGE_BOOT_SETUP))
        zerOS_hcf(); */

    if (LIMINE_BASE_REVISION_SUPPORTED == false)
        zerOS_hcf();

    if (!assert_uefi_x86_64())
        zerOS_hcf();

    if (!setup_early_debug())
        zerOS_hcf();

    zerOS_early_printk("zerOS: loading and setting up eventual kernel modules\n");
    if (!setup_kern_modules())
        zerOS_hcf();

    zerOS_early_printk("zerOS: setting up GDT\n");
    if (!setup_gdt())
        zerOS_hcf();

    zerOS_early_printk("zerOS: setting up paging\n");
    if (!setup_paging())
        zerOS_hcf();

    zerOS_early_printk("zerOS: setting up ISA extensions\n");
    if (!setup_isa_exts())
        zerOS_hcf();

    zerOS_early_printk("zerOS: setting up IDT\n");
    if (!setup_idt())
        zerOS_hcf();

    if (framebuffer_request.response == nullptr
     || framebuffer_request.response->framebuffer_count < 1) {
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: jumping to kernel main\n");
    extern void zerOS_kmain(void);
    zerOS_kmain();
}