#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#include <limine.h>

#include <misc/sections.h>

#include <klibc/maybe.h>

static uintptr_t get_cr3(void)
{
    uintptr_t ret;
    asm volatile(
        "mov %%cr3, %0" : "=r"(ret)
    );
    return ret;
}

static uintptr_t virt_to_phys(uintptr_t virt)
{
    uint64_t pml4e = ((virt >> 39) & 0x1FF);
    uint64_t pdpe = ((virt >> 30) & 0x1FF);
    uint64_t pde = ((virt >> 21) & 0x1FF);
    uint64_t pte = ((virt >> 12) & 0x1FF);

    uint64_t* pml4 = (uint64_t*) get_cr3();
    uint64_t* pdp = (uint64_t*) (pml4[pml4e] & 0xFFFFFFFFFFFFF000);
    uint64_t* pd = (uint64_t*) (pdp[pdpe] & 0xFFFFFFFFFFFFF000);
    uint64_t* pt = (uint64_t*) (pd[pde] & 0xFFFFFFFFFFFFF000);

    return (pt[pte] & 0xFFFFFFFFFFFFF000) | (virt & 0xFFF);
}