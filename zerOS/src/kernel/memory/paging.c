#include <config.h>

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#include <limine.h>

#include <klibc/maybe.h>

#include <kernel/limine_data.h>
#include <kernel/memory/paging.h>
#include <kernel/memory/pmm.h>

#include <misc/sections.h>
#include <misc/symbol.h>

extern bool zerOS_init_early_paging(void)
{
    struct limine_memmap_response* memmap_resp =
      (struct limine_memmap_response*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_RESPONSE);
    struct limine_memmap_entry* memmap =
      (struct limine_memmap_entry*)zerOS_get_limine_data(zerOS_LIMINE_MEMMAP_ENTRY, (size_t)0);
    const size_t memmap_entry_count = memmap_resp->entry_count;

    struct limine_hhdm_response* hhdm =
      (struct limine_hhdm_response*)zerOS_get_limine_data(zerOS_LIMINE_HHDM_RESPONSE);

    struct limine_kernel_address_response* kernaddr = (struct limine_kernel_address_response*)
      zerOS_get_limine_data(zerOS_LIMINE_KERNEL_ADDRESS_RESPONSE);
}
