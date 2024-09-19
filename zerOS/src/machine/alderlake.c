#include <config.h>

#include <stdint.h>
#include <x86gprintrin.h>

#include <machine/setup.h>

BOOT_FUNC
static uint64_t read_cr0(void)
{
    uint64_t cr0;
    asm volatile(
        "mov %%cr0, %0"
        : "=r"(cr0)
    );
    return cr0;
}

BOOT_FUNC
static uint64_t read_cr3(void)
{
    uint64_t cr3;
    asm volatile(
        "mov %%cr3, %0"
        : "=r"(cr3)
    );
    return cr3;
}

BOOT_FUNC
static uint64_t read_cr4(void)
{
    uint64_t cr4;
    asm volatile(
        "mov %%cr4, %0"
        : "=r"(cr4)
    );
    return cr4;
}

BOOT_FUNC
static void write_cr0(uint64_t cr0)
{
    asm volatile(
        "mov %0, %%cr0"
        :
        : "r"(cr0)
    );
}

BOOT_FUNC
static void write_cr3(uint64_t cr3)
{
    asm volatile(
        "mov %0, %%cr3"
        :
        : "r"(cr3)
    );
}

BOOT_FUNC
static void write_cr4(uint64_t cr4)
{
    asm volatile(
        "mov %0, %%cr4"
        :
        : "r"(cr4)
    );
}

BOOT_FUNC
static uint64_t read_xcr0(void)
{ return _xgetbv(0); }

BOOT_FUNC
static void write_xcr0(uint64_t xcr0)
{ _xsetbv(0, xcr0); }

BOOT_FUNC
extern bool alderlake_setup_isa_exts(void)
{
    uint64_t cr0  = read_cr0();
    uint64_t cr4  = read_cr4();
    uint64_t xcr0 = read_xcr0();

    // Set CR4.OSFXSR[bit 9] = 1
    cr4 |= (1ULL << 9);
    // Set CR4.OSXMMEXCPT[bit 10] = 1
    cr4 |= (1ULL << 10);
    // Clear CR0.EM[bit 2] = 0
    cr0 &= ~(1ULL << 2);
    // Set CR0.MP[bit 1] = 1
    cr0 |= (1ULL << 1);
    // Set CR4.OSXSAVE[bit 18] = 1
    cr4 |= (1ULL << 18);
    
    // Set XCR0[bit 0] = 1 (X87)
    // Set XCR0[bit 1] = 1 (SSE)
    // Set XCR0[bit 2] = 1 (AVX)
    xcr0 |= 1ULL | (1ULL << 1) | (1ULL << 2);

    write_cr0(cr0);
    write_cr4(cr4);
    write_xcr0(xcr0);

    return true;
}