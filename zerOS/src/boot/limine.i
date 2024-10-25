/*
 * limine.i
 * Do not include this file anywhere else than in `zerOS/src/boot/limine_requests.c`.
 * Do not compile this file directly either.
 * Limine requests have been placed here for the sake of clarity / lisibility.
 */


IN_SECTION(".requests") SYMBOL_USED
static LIMINE_BASE_REVISION(LIMINE_REQUESTED_REVISION);

// The Limine requests can be placed anywhere, but it is important that
// the compiler does not optimise them away, so, usually, they should
// be made volatile or equivalent, _and_ they should be accessed at least
// once or marked as used with the "used" attribute as done here.

// Ask Limine for 5LVL paging
IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_paging_mode_request lvl5_paging_request = {
    .id = LIMINE_PAGING_MODE_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr,
    .max_mode = LIMINE_PAGING_MODE_X86_64_5LVL,
    .min_mode = LIMINE_PAGING_MODE_X86_64_4LVL
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_framebuffer_request framebuffer_request = {
    .id = LIMINE_FRAMEBUFFER_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_firmware_type_request firmware_type_request = {
    .id = LIMINE_FIRMWARE_TYPE_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_hhdm_request hhdm_request = {
    .id = LIMINE_HHDM_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_memmap_request memmap_request = {
    .id = LIMINE_MEMMAP_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_efi_memmap_request efi_memmap_request = {
    .id = LIMINE_EFI_MEMMAP_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_efi_system_table_request efi_system_table_request = {
    .id = LIMINE_EFI_SYSTEM_TABLE_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};

IN_SECTION(".requests") SYMBOL_USED
static volatile struct limine_kernel_address_request kernel_address_request = {
    .id = LIMINE_KERNEL_ADDRESS_REQUEST,
    .revision = LIMINE_REQUESTED_REVISION,
    .response = nullptr
};


// Finally, define the start and end markers for the Limine requests.
// These can also be moved anywhere, to any .c file, as seen fit.

IN_SECTION(".requests_start_marker") SYMBOL_USED
static LIMINE_REQUESTS_START_MARKER;

IN_SECTION(".requests_end_marker") SYMBOL_USED
static LIMINE_REQUESTS_END_MARKER;