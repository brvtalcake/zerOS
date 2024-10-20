#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdarg.h>

#include <limine.h>

#include <kernel/printk.h>
#include <kernel/limine_data.h>
#include <kernel/cpu/misc.h>
#include <kernel/cpu/io.h>

#include <misc/sections.h>
#include <misc/symbol.h>

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
            size_t video_mode_id = va_arg(args, size_t);
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

        default: return nullptr;
    }
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

#undef  __assert_good_response
#define __assert_good_response(request, additionalchecks)   \
    if ((request).response == nullptr ||                    \
        (request).revision != LIMINE_REQUESTED_REVISION ||  \
        !(additionalchecks))                                \
    {                                                       \
        zerOS_early_printk(                                 \
            "zerOS: no good response for request `%s`\n",   \
            #request                                        \
        );                                                  \
        zerOS_hcf();                                        \
    }

BOOT_FUNC
static void copy_framebuffers(void)
{
    if (framebuffer_response.framebuffer_count > __MAX_FB_COUNT)
    {
        zerOS_early_printk(
            "zerOS: framebuffer count is too high (%u)\n",
            (unsigned int) framebuffer_response.framebuffer_count
        );
        zerOS_hcf();
    }
    for (size_t i = 0; i < framebuffer_response.framebuffer_count; i++)
    {
        boot_memcpy(framebuffers + i, *(framebuffer_response.framebuffers + i), sizeof(struct limine_framebuffer));
        
        if (framebuffers[i].mode_count > __MAX_FB_VIDEO_MODE_COUNT)
        {
            zerOS_early_printk(
                "zerOS: video mode count is too high (%u)\n",
                (unsigned int) framebuffers[i].mode_count
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
    if (memmap_response.entry_count > __MAX_MEMMAP_ENTRY_COUNT)
    {
        zerOS_early_printk(
            "zerOS: memmap entry count is too high (%u)\n",
            (unsigned int) memmap_response.entry_count
        );
        zerOS_hcf();
    }
    for (size_t i = 0; i < memmap_response.entry_count; i++)
        boot_memcpy(memmap_entry_buf + i, *(memmap_response.entries + i), sizeof(struct limine_memmap_entry));
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

    __assert_good_response(lvl5_paging_request, true);
    boot_memcpy(&paging_response, lvl5_paging_request.response, sizeof(paging_response));

    __assert_good_response(framebuffer_request, true);
    boot_memcpy(&framebuffer_response, framebuffer_request.response, sizeof(framebuffer_response));
    copy_framebuffers();

    __assert_good_response(firmware_type_request, true);
    boot_memcpy(&firmware_type_response, firmware_type_request.response, sizeof(firmware_type_response));

    __assert_good_response(hhdm_request, true);
    boot_memcpy(&hhdm_response, hhdm_request.response, sizeof(hhdm_response));

    __assert_good_response(memmap_request, true);
    boot_memcpy(&memmap_response, memmap_request.response, sizeof(memmap_response));
    copy_memmap_entries();

    __assert_good_response(efi_memmap_request, true);
    boot_memcpy(&efi_memmap_response, efi_memmap_request.response, sizeof(efi_memmap_response));

    __assert_good_response(efi_system_table_request, true);
    boot_memcpy(&efi_system_table_response, efi_system_table_request.response, sizeof(efi_system_table_response));
}