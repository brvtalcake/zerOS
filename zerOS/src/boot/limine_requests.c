#include <config.h>

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <limine.h>

#include <kernel/cpu/io.h>
#include <kernel/cpu/misc.h>
#include <kernel/limine_data.h>
#include <kernel/printk.h>

#include <misc/sections.h>
#include <misc/statement.h>
#include <misc/symbol.h>

// clang-format off
#ifdef LIMINE_REQUESTED_REVISION
    #error "LIMINE_REQUESTED_REVISION shall not be defined before this point"
#endif
#define LIMINE_REQUESTED_REVISION (UINT64_C(2))

#include "limine.i"

#undef  __MAX_FB_COUNT
#undef  __MAX_FB_VIDEO_MODE_COUNT
#define __MAX_FB_COUNT 16
#define __MAX_FB_VIDEO_MODE_COUNT 128

#undef  __MAX_MEMMAP_ENTRY_COUNT
#define __MAX_MEMMAP_ENTRY_COUNT 256
// clang-format on

static struct limine_paging_mode_response paging_response;

static struct limine_framebuffer_response framebuffer_response;
static struct limine_framebuffer          framebuffers[__MAX_FB_COUNT];
static struct limine_video_mode           video_modes[__MAX_FB_COUNT][__MAX_FB_VIDEO_MODE_COUNT];

static struct limine_firmware_type_response firmware_type_response;

static struct limine_hhdm_response hhdm_response;

static struct limine_memmap_response memmap_response;
static struct limine_memmap_entry    memmap_entry_buf[__MAX_MEMMAP_ENTRY_COUNT];

static struct limine_efi_memmap_response efi_memmap_response;

static struct limine_efi_system_table_response efi_system_table_response;

static struct limine_kernel_address_response kernel_address_response;

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
        case LIMINE_MEMMAP_USABLE:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_RESERVED:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_ACPI_RECLAIMABLE:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_ACPI_NVS:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_BAD_MEMORY:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_KERNEL_AND_MODULES:
            CASE_FALLTHROUGH;
        case LIMINE_MEMMAP_FRAMEBUFFER:
            return strings[type];
        default:
            return unknown;
    }
};

BOOT_FUNC
static void print_entries(struct limine_memmap_entry* entries, size_t entry_count)
{
    if (entries && entry_count)
    {
        for (size_t i = 0; i < entry_count; i++)
        {
            struct limine_memmap_entry entry = entries[i];
            zerOS_early_printk(
              "zerOS: entry %u: base = 0x%p, length = 0x%x, type = %s\n", EPRI_CAST(u, i),
              EPRI_CAST(p, entry.base), EPRI_CAST(x, entry.length),
              EPRI_CAST(s, limine_entry_type_string(entry.type))
            );
        }
    }
    else
    {
        struct limine_memmap_response* response =
          (struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
        for (size_t i = 0; i < response->entry_count; i++)
        {
            struct limine_memmap_entry* entry = zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, i);
            zerOS_early_printk(
              "zerOS: entry %u: base = 0x%p, length = 0x%x, type = %s\n", EPRI_CAST(u, i),
              EPRI_CAST(p, entry->base), EPRI_CAST(x, entry->length),
              EPRI_CAST(s, limine_entry_type_string(entry->type))
            );
        }
    }
};

BOOT_FUNC
extern void* zerOS_get_limine_data(enum zerOS_limine_data_request req, ...)
{
    switch (req)
    {
        case zerOS_LIMINE_PAGING_RESPONSE:
            return &paging_response;

        case zerOS_LIMINE_FRAMEBUFFER_RESPONSE:
            return &framebuffer_response;

        case zerOS_LIMINE_FIRMWARE_TYPE_RESPONSE:
            return &firmware_type_response;

        case zerOS_LIMINE_HHDM_RESPONSE:
            return &hhdm_response;

        case zerOS_LIMINE_MEMMAP_RESPONSE:
            return &memmap_response;

        case zerOS_LIMINE_EFI_MEMMAP_RESPONSE:
            return &efi_memmap_response;

        case zerOS_LIMINE_EFI_SYSTEM_TABLE_RESPONSE:
            return &efi_system_table_response;

        case zerOS_LIMINE_KERNEL_ADDRESS_RESPONSE:
            return &kernel_address_response;

        case zerOS_LIMINE_FRAMEBUFFER: {
            va_list args;
            va_start(args, req);
            size_t framebuffer_id = va_arg(args, size_t);
            va_end(args);
            if (framebuffer_id >= framebuffer_response.framebuffer_count)
                return nullptr;
            return framebuffers + framebuffer_id;
        };

        case zerOS_LIMINE_FB_VIDEO_MODE: {
            va_list args;
            va_start(args, req);
            size_t framebuffer_id = va_arg(args, size_t);
            size_t video_mode_id  = va_arg(args, size_t);
            va_end(args);
            if (framebuffer_id >= framebuffer_response.framebuffer_count ||
                video_mode_id >= framebuffers[framebuffer_id].mode_count)
                return nullptr;
            return video_modes[framebuffer_id] + video_mode_id;
        };

        case zerOS_LIMINE_MEMMAP_ENTRY: {
            va_list args;
            va_start(args, req);
            size_t entry_id = va_arg(args, size_t);
            va_end(args);
            if (entry_id >= memmap_response.entry_count)
                return nullptr;
            return memmap_entry_buf + entry_id;
        };

        default:
            return nullptr;
    }
}

BOOT_FUNC
static inline void* boot_memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    unsigned char*       d = dest;
    const unsigned char* s = src;
    while (n--)
        *d++ = *s++;
    return dest;
}

// clang-format off
#undef  __assert_good_response
// clang-format on
#define __assert_good_response(request, additionalchecks)                                         \
    if ((request).response == nullptr ||                                                          \
        (request).revision != LIMINE_REQUESTED_REVISION ||                                        \
        !(additionalchecks))                                                                      \
    {                                                                                             \
        zerOS_early_printk("zerOS: no good response for request `%s`\n", EPRI_CAST(s, #request)); \
        zerOS_hcf();                                                                              \
    }

BOOT_FUNC
static void copy_framebuffers(void)
{
    if (framebuffer_response.framebuffer_count > __MAX_FB_COUNT)
    {
        zerOS_early_printk(
          "zerOS: framebuffer count is too high (%u)\n",
          EPRI_CAST(u, framebuffer_response.framebuffer_count)
        );
        zerOS_hcf();
    }
    for (size_t i = 0; i < framebuffer_response.framebuffer_count; i++)
    {
        boot_memcpy(
          framebuffers + i, *(framebuffer_response.framebuffers + i),
          sizeof(struct limine_framebuffer)
        );
        zerOS_early_printk(
          "zerOS: framebuffer %u: address = 0x%p\n", EPRI_CAST(u, i),
          EPRI_CAST(p, framebuffers[i].address)
        );

        if (framebuffers[i].mode_count > __MAX_FB_VIDEO_MODE_COUNT)
        {
            zerOS_early_printk(
              "zerOS: video mode count is too high (%u)\n", EPRI_CAST(u, framebuffers[i].mode_count)
            );
            zerOS_hcf();
        }
        struct limine_video_mode** _video_modes = framebuffers[i].modes;
        for (size_t j = 0; j < framebuffers[i].mode_count; j++)
            boot_memcpy(video_modes[i] + j, *(_video_modes + j), sizeof(struct limine_video_mode));
    }
}

BOOT_FUNC
static void copy_memmap_entries(void)
{
    size_t usable_count = 0;
    if (memmap_response.entry_count > __MAX_MEMMAP_ENTRY_COUNT)
    {
        zerOS_early_printk(
          "zerOS: memmap entry count is too high (%u)\n", EPRI_CAST(u, memmap_response.entry_count)
        );
        zerOS_hcf();
    }
    for (size_t i = 0; i < memmap_response.entry_count; i++)
    {
        boot_memcpy(
          memmap_entry_buf + i, *(memmap_response.entries + i), sizeof(struct limine_memmap_entry)
        );
        if (memmap_entry_buf[i].type == LIMINE_MEMMAP_USABLE ||
            memmap_entry_buf[i].type == LIMINE_MEMMAP_BOOTLOADER_RECLAIMABLE)
            usable_count++;
    }
    if (unlikely(usable_count == 0))
    {
        zerOS_early_printk("zerOS: no usable memory regions found\n");
        zerOS_hcf();
    }
    if (unlikely(usable_count > zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS))
    {
        zerOS_early_printk(
          "zerOS: too many usable memory regions found (%u)\n", EPRI_CAST(u, usable_count)
        );
        zerOS_hcf();
    }
}

BOOT_FUNC
extern void zerOS_copy_limine_requests(void)
{
    zerOS_early_printk("zerOS: copying Limine requests\n");

    if (!LIMINE_BASE_REVISION_SUPPORTED)
    {
        zerOS_early_printk("zerOS: Limine base revision is not supported\n");
        zerOS_hcf();
    }

    __assert_good_response(
      lvl5_paging_request, lvl5_paging_request.response->mode == LIMINE_PAGING_MODE_X86_64_5LVL ||
                             lvl5_paging_request.response->mode == LIMINE_PAGING_MODE_X86_64_4LVL
    );
    boot_memcpy(&paging_response, lvl5_paging_request.response, sizeof(paging_response));

    __assert_good_response(
      framebuffer_request, framebuffer_request.response->framebuffer_count > 0 &&
                             framebuffer_request.response->framebuffers != nullptr
    );
    boot_memcpy(&framebuffer_response, framebuffer_request.response, sizeof(framebuffer_response));
    copy_framebuffers();

    __assert_good_response(firmware_type_request, true);
    boot_memcpy(
      &firmware_type_response, firmware_type_request.response, sizeof(firmware_type_response)
    );

    __assert_good_response(hhdm_request, true);
    boot_memcpy(&hhdm_response, hhdm_request.response, sizeof(hhdm_response));
    zerOS_early_printk(
      "zerOS: kernel HHDM = 0x%x\n",
      EPRI_CAST(
        x, ((struct limine_hhdm_response*)zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE))->offset
      )
    );

    __assert_good_response(memmap_request, true);
    boot_memcpy(&memmap_response, memmap_request.response, sizeof(memmap_response));
    copy_memmap_entries();
    print_entries(nullptr, 0);

    __assert_good_response(efi_memmap_request, true);
    boot_memcpy(&efi_memmap_response, efi_memmap_request.response, sizeof(efi_memmap_response));

    __assert_good_response(efi_system_table_request, true);
    boot_memcpy(
      &efi_system_table_response, efi_system_table_request.response,
      sizeof(efi_system_table_response)
    );

    __assert_good_response(kernel_address_request, true);
    boot_memcpy(
      &kernel_address_response, kernel_address_request.response, sizeof(kernel_address_response)
    );
    zerOS_early_printk(
      "zerOS: kernel address base: physical = 0x%p, virtual = 0x%p\n",
      EPRI_CAST(p, kernel_address_response.physical_base),
      EPRI_CAST(p, kernel_address_response.virtual_base)
    );
}
