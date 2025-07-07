#ifdef __INTELLISENSE__
#	define zerOS_INIT_BOOTLOADER_IS_LIMINE 1
#endif

// leaf 0x1
// EDX
/* do we need this one ? */
/* HAVE_CLFLUSH_EDX_MASK  equ 0x1ULL << 19 */
#define HAVE_FXSAVE_EDX_MASK (0x1ULL << 24)
#define HAVE_SSE_EDX_MASK    (0x1ULL << 25)
#define HAVE_SSE2_EDX_MASK   (0x1ULL << 26)
// ECX
#define HAVE_SSE3_ECX_MASK  (0x1ULL << 0)
#define HAVE_SSSE3_ECX_MASK (0x1ULL << 9)
#define HAVE_SSE41_ECX_MASK (0x1ULL << 19)
#define HAVE_SSE42_ECX_MASK (0x1ULL << 20)
#define HAVE_XSAVE_ECX_MASK (0x1ULL << 26)
#define HAVE_AVX_ECX_MASK   (0x1ULL << 28)

// leaf 0x7, sub-leaf 0x0
// EBX
#define HAVE_AVX2_EBX_MASK (0x1ULL << 5)

// leaf 0xd, sub-leaf 0x0
// EAX
/* TODO: is that sufficient ? */
#define HAVE_AVX512_EAX_MASK (0x1ULL << 5 | 0x1ULL << 6 | 0x1ULL << 7)

#define HAVE_FXSAVE (0x1ULL << 0)
#define HAVE_XSAVE  (0x1ULL << 1)
#define HAVE_SSE    (0x1ULL << 2)
#define HAVE_SSE2   (0x1ULL << 3)
#define HAVE_SSE3   (0x1ULL << 4)
#define HAVE_SSSE3  (0x1ULL << 5)
#define HAVE_SSE41  (0x1ULL << 6)
#define HAVE_SSE42  (0x1ULL << 7)
#define HAVE_AVX    (0x1ULL << 8)
#define HAVE_AVX2   (0x1ULL << 9)
#define HAVE_AVX512 (0x1ULL << 10)

#include <cpuid.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <x86intrin.h>

static uint16_t feature_buffer;

static inline bool can_enable_sse(void)
{
	return (feature_buffer
			& (HAVE_SSE | HAVE_SSE2 | HAVE_SSE3 | HAVE_SSSE3 | HAVE_SSE41 | HAVE_SSE42))
		&& (feature_buffer & (HAVE_FXSAVE | HAVE_XSAVE));
}

static inline bool can_enable_avx(void)
{
	return can_enable_sse()
		&& (feature_buffer & (HAVE_AVX | HAVE_AVX2))
		&& (feature_buffer & HAVE_XSAVE);
}

static inline bool can_enable_avx512(void)
{
	return can_enable_avx() && (feature_buffer & HAVE_AVX512);
}

[[__gnu__::__section__(".bootcode")]]
static void detect_features(void)
{
	uint32_t eax, ebx, ecx, edx;
	feature_buffer = 0;

	eax = 0;
	ebx = 0;
	ecx = 0;
	edx = 0;
	if (!__get_cpuid_count(0x1, 0x0, &eax, &ebx, &ecx, &edx))
		goto next_check1;

	if ((ecx & HAVE_SSE3_ECX_MASK) == HAVE_SSE3_ECX_MASK)
		feature_buffer |= HAVE_SSE3;
	if ((ecx & HAVE_SSSE3_ECX_MASK) == HAVE_SSSE3_ECX_MASK)
		feature_buffer |= HAVE_SSSE3;
	if ((ecx & HAVE_SSE41_ECX_MASK) == HAVE_SSE41_ECX_MASK)
		feature_buffer |= HAVE_SSE41;
	if ((ecx & HAVE_SSE42_ECX_MASK) == HAVE_SSE42_ECX_MASK)
		feature_buffer |= HAVE_SSE42;
	if ((ecx & HAVE_XSAVE_ECX_MASK) == HAVE_XSAVE_ECX_MASK)
		feature_buffer |= HAVE_XSAVE;
	if ((ecx & HAVE_AVX_ECX_MASK) == HAVE_AVX_ECX_MASK)
		feature_buffer |= HAVE_AVX;

	if ((edx & HAVE_FXSAVE_EDX_MASK) == HAVE_FXSAVE_EDX_MASK)
		feature_buffer |= HAVE_FXSAVE;
	if ((edx & HAVE_SSE_EDX_MASK) == HAVE_SSE_EDX_MASK)
		feature_buffer |= HAVE_SSE;
	if ((edx & HAVE_SSE2_EDX_MASK) == HAVE_SSE2_EDX_MASK)
		feature_buffer |= HAVE_SSE2;

next_check1:
	eax = 0;
	ebx = 0;
	ecx = 0;
	edx = 0;
	if (!__get_cpuid_count(0x7, 0x0, &eax, &ebx, &ecx, &edx))
		goto next_check2;

	if ((ebx & HAVE_AVX2_EBX_MASK) == HAVE_AVX2_EBX_MASK)
		feature_buffer |= HAVE_AVX2;

next_check2:
	eax = 0;
	ebx = 0;
	ecx = 0;
	edx = 0;
	if (!__get_cpuid_count(0xd, 0x0, &eax, &ebx, &ecx, &edx))
		return;

	if ((eax & HAVE_AVX512_EAX_MASK) == HAVE_AVX512_EAX_MASK)
		feature_buffer |= HAVE_AVX512;
}

[[__gnu__::__section__(".bootcode")]]
static inline uint64_t read_cr0(void)
{
	uint64_t cr0;
	asm volatile("mov %%cr0, %0"
				 : "=r"(cr0));
	return cr0;
}

[[__gnu__::__section__(".bootcode")]]
static inline uint64_t read_cr4(void)
{
	uint64_t cr4;
	asm volatile("mov %%cr4, %0"
				 : "=r"(cr4));
	return cr4;
}

[[__gnu__::__section__(".bootcode")]]
static inline void write_cr0(uint64_t cr0)
{
	asm volatile("mov %0, %%cr0"
				 :
				 : "r"(cr0));
}

[[__gnu__::__section__(".bootcode")]]
static inline void write_cr4(uint64_t cr4)
{
	asm volatile("mov %0, %%cr4"
				 :
				 : "r"(cr4));
}

[[__gnu__::__section__(".bootcode")]]
static inline uint64_t read_xcr0(void)
{
	uint32_t xcr0_hi, xcr0_lo;
	asm volatile("xor %%rcx, %%rcx\n"
				 "xgetbv\n"
				 : "=a"(xcr0_lo), "=d"(xcr0_hi)
				 :
				 : "rcx");
	return ((uint64_t)xcr0_hi << 32) | xcr0_lo;
}

[[__gnu__::__section__(".bootcode")]]
static inline void write_xcr0(uint64_t xcr0)
{
	const uint32_t xcr0_hi = (xcr0 >> 32) & UINT32_MAX;
	const uint32_t xcr0_lo = xcr0 & UINT32_MAX;
	asm volatile("xor %%rcx, %%rcx\n"
				 "xsetbv\n"
				 :
				 : "a"(xcr0_lo), "d"(xcr0_hi)
				 : "rcx");
}

[[__gnu__::__section__(".bootcode")]] [[__noreturn__]]
extern void
#if zerOS_INIT_BOOTLOADER_IS_LIMINE
zerOS_entry_point(void)
#elif zerOS_INIT_BOOTLOADER_IS_GRUB2
#	error "this is yet to be done"
#elif zerOS_INIT_BOOTLOADER_IS_UEFI
#	error "this is yet to be done"
#else
#	error "no bootloader has been defined"
#endif
{
	detect_features();

	// probably already enabled by the firmware or the bootloader, but at least we avoid any bad
	// surprises
	if (can_enable_sse())
	{
		uint64_t cr0 = read_cr0();
		uint64_t cr4 = read_cr4();

		// Clear CR0.EM[bit 2] = 0
		cr0 &= ~(1ULL << 2);
		// Set CR0.MP[bit 1] = 1
		cr0 |= (1ULL << 1);
		// Set CR4.OSFXSR[bit 9] = 1
		cr4 |= (1ULL << 9);
		// Set CR4.OSXMMEXCPT[bit 10] = 1
		cr4 |= (1ULL << 10);
		// Set CR4.OSXSAVE[bit 18] = 1
		if (feature_buffer & HAVE_XSAVE)
			cr4 |= (1ULL << 18);

		write_cr0(cr0);
		write_cr4(cr4);
	}

	if (can_enable_avx())
	{
		uint64_t xcr0 = read_xcr0();

		// Set XCR0[bit 0] = 1 (X87 bit)
		// Set XCR0[bit 1] = 1 (SSE bit)
		// Set XCR0[bit 2] = 1 (AVX bit)
		xcr0 |= 1ULL | (1ULL << 1) | (1ULL << 2);

		// Set XCR0[bit 5] = 1 (OPMASK bit)
		// Set XCR0[bit 6] = 1 (ZMM_Hi256 bit)
		// Set XCR0[bit 7] = 1 (Hi16_ZMM bit)
		if (can_enable_avx512())
			xcr0 |= (1ULL << 5) | (1ULL << 6) | (1ULL << 7);

		write_xcr0(xcr0);
	}

#if zerOS_INIT_BOOTLOADER_IS_LIMINE
	extern void zerOS_boot_setup(void);
	zerOS_boot_setup();
#elif zerOS_INIT_BOOTLOADER_IS_GRUB2
#	error "this is yet to be done"
#elif zerOS_INIT_BOOTLOADER_IS_UEFI
#	error "this is yet to be done"
#else
#	error "no bootloader has been defined"
#endif

	unreachable();
}
