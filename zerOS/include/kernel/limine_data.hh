#ifndef zerOS_KERNEL_LIMINE_DATA_H_INCLUDED
#define zerOS_KERNEL_LIMINE_DATA_H_INCLUDED

#include <misc/sections.h>

enum zerOS_limine_data_request
{
    zerOS_LIMINE_PAGING_RESPONSE,
    zerOS_LIMINE_FRAMEBUFFER_RESPONSE,
    zerOS_LIMINE_FIRMWARE_TYPE_RESPONSE,
    zerOS_LIMINE_HHDM_RESPONSE,
    zerOS_LIMINE_MEMMAP_RESPONSE,
    zerOS_LIMINE_EFI_MEMMAP_RESPONSE,
    zerOS_LIMINE_EFI_SYSTEM_TABLE_RESPONSE,
    zerOS_LIMINE_KERNEL_ADDRESS_RESPONSE,
    zerOS_LIMINE_FRAMEBUFFER,
    zerOS_LIMINE_FB_VIDEO_MODE,
    zerOS_LIMINE_MEMMAP_ENTRY
};

BOOT_FUNC
extern void zerOS_copy_limine_requests(void);
BOOT_FUNC
extern void* zerOS_get_limine_data(enum zerOS_limine_data_request req, ...);
BOOT_FUNC
extern bool zerOS_has_bootloaded_modules(void);

#endif
