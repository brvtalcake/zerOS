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
#include <misc/func.h>

#include <kernel/printk.h>
#include <kernel/limine_data.h>
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
        char* UNIQUE(ptr) = __builtin_alloca(size); \
        for (                                       \
            size_t UNIQUE(i) = 0;                   \
            UNIQUE(i) < size;                       \
            UNIQUE(i)++                             \
        ) UNIQUE(ptr)[UNIQUE(i)] = 0;               \
        (void*)UNIQUE(ptr);                         \
    })

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
    struct limine_memmap_response* response = (struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    if (!response)
        return 0;
    size_t count = 0;
    for (size_t i = 0; i < response->entry_count; i++)
    {
        struct limine_memmap_entry* entry = (struct limine_memmap_entry*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
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
    struct limine_firmware_type_response* response = (struct limine_firmware_type_response*)zerOS_get_limine_data(zerOS_LIMINE_FIRMWARE_TYPE_RESPONSE);
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
static size_t get_available_physical_memory(void)
{
    return 0;
}

BOOT_FUNC
static char* limine_entry_type_string(uint64_t type)
{
    static char strings[8][32] = {
        [LIMINE_MEMMAP_USABLE]                 = "USABLE\0",
        [LIMINE_MEMMAP_RESERVED]               = "RESERVED\0",
        [LIMINE_MEMMAP_ACPI_RECLAIMABLE]       = "ACPI_RECLAIMABLE\0",
        [LIMINE_MEMMAP_ACPI_NVS]               = "ACPI_NVS\0",
        [LIMINE_MEMMAP_BAD_MEMORY]             = "BAD_MEMORY\0",
        [LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE] = "BOOTLOADER_RECLAIMABLE\0",
        [LIMINE_MEMMAP_KERNEL_AND_MODULES]     = "KERNEL_AND_MODULES\0",
        [LIMINE_MEMMAP_FRAMEBUFFER]            = "FRAMEBUFFER\0"
    };
    static char unknown[32] = "UNKNOWN\0";
    switch (type)
    {
        case LIMINE_MEMMAP_USABLE: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_RESERVED: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_ACPI_RECLAIMABLE: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_ACPI_NVS: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_BAD_MEMORY: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_KERNEL_AND_MODULES: CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_FRAMEBUFFER: return strings[type];
        default: return unknown;
    }
};

BOOT_FUNC
static void print_entries(struct limine_memmap_entry* entries, size_t entry_count)
{
    if (entries)
    {
        for (size_t i = 0; i < entry_count; i++)
        {
            struct limine_memmap_entry entry = entries[i];
            zerOS_early_printk("zerOS: entry %u: base = 0x%x, length = 0x%x, type = %s\n",
                (unsigned int)i, entry.base, entry.length, limine_entry_type_string(entry.type));
        }
    }
    else
    {
        struct limine_memmap_response* response = (struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
        for (size_t i = 0; i < response->entry_count; i++)
        {
            struct limine_memmap_entry* entry = zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
            zerOS_early_printk("zerOS: entry %u: base = 0x%x, length = 0x%x, type = %s\n",
                (unsigned int)i, entry->base, entry->length, limine_entry_type_string(entry->type));
        }
    }
};

BOOT_FUNC
static bool setup_paging(void)
{
    print_entries(nullptr, 0);

    zerOS_init_paging_values();

    struct limine_paging_mode_response* response = zerOS_get_limine_data(zerOS_LIMINE_PAGING_RESPONSE);

    if (
        response->mode != LIMINE_PAGING_MODE_X86_64_5LVL &&
        response->mode != LIMINE_PAGING_MODE_X86_64_4LVL
    )
        return false;

    struct limine_memmap_entry* entry_buf = calloca(
        sizeof(struct limine_memmap_entry) * ((struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE))->entry_count
    );
    const size_t entry_count = get_needed_mem_entries(entry_buf);

    if (entry_count == 0)
        return false;

    print_entries(entry_buf, entry_count);

    zerOS_early_printk("zerOS: kernel HHDM = 0x%x\n", (unsigned int) ((struct limine_hhdm_response*) zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE))->offset);

    //if (!zerOS_init_pmm(entry_buf, entry_count))
    //    return false;

    //if (!zerOS_init_vmm())
    //    return false;

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

BOOT_FUNC FUNC_NORETURN
extern void zerOS_boot_setup(void)
{
    //if (!zerOS_stage_set(zerOS_STAGE_BOOT_SETUP))
    //    zerOS_hcf();

    if (!setup_early_debug())
        zerOS_hcf();
    
    zerOS_early_printk("zerOS: setting up ISA extensions\n");
    if (!setup_isa_exts())
    {
        zerOS_early_printk("zerOS: failed to setup ISA extensions\n");
        zerOS_hcf();
    }

    zerOS_copy_limine_requests();
    
    if (!assert_uefi_x86_64())
    {
        zerOS_early_printk("zerOS: not running on UEFI x86_64\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: setting up paging\n");
    if (!setup_paging())
    {
        zerOS_early_printk("zerOS: failed to setup paging\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: loading and setting up eventual kernel modules\n");
    if (!setup_kern_modules())
    {
        zerOS_early_printk("zerOS: failed to load and setup kernel modules\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: setting up GDT\n");
    if (!setup_gdt())
    {
        zerOS_early_printk("zerOS: failed to setup GDT\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: setting up IDT\n");
    if (!setup_idt())
    {
        zerOS_early_printk("zerOS: failed to setup IDT\n");
        zerOS_hcf();
    }

    zerOS_early_printk("zerOS: jumping to kernel main\n");
    extern void zerOS_kmain(void);
    zerOS_kmain();

    zerOS_hcf();
}