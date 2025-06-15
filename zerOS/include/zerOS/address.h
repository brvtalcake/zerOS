#ifndef zerOS_ADDRESS_H_INCLUDED_
#define zerOS_ADDRESS_H_INCLUDED_ 1

#include <stdint.h>

#include <zerOS/common.h>

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_physaddr_zero_extend(uintptr_t addr)
{
	extern size_t zerOS_boot_cpu_physical_address_bits;
	assert(
	  zerOS_boot_cpu_physical_address_bits >= 32
	  && zerOS_boot_cpu_physical_address_bits <= MAX_VIRTUAL_ADDRESS_LOG2);

	const size_t mask = ((uintptr_t)1 << zerOS_boot_cpu_physical_address_bits) - 1;
	return addr & mask;
}

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_physaddr_sign_extend(uintptr_t addr)
{
	extern size_t zerOS_boot_cpu_physical_address_bits;
	assert(
	  zerOS_boot_cpu_physical_address_bits >= 32
	  && zerOS_boot_cpu_physical_address_bits <= MAX_VIRTUAL_ADDRESS_LOG2);

	const size_t shift = UINTPTR_WIDTH - zerOS_boot_cpu_physical_address_bits;
	return (uintptr_t)((intptr_t)(addr << shift) >> shift);
}

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_physaddr_canonicalize(uintptr_t addr)
{
	return zerOS_physaddr_zero_extend(addr);
}

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_virtaddr_zero_extend(uintptr_t addr)
{
	extern size_t zerOS_boot_cpu_linear_address_bits;
	assert(
	  zerOS_boot_cpu_linear_address_bits >= 32
	  && zerOS_boot_cpu_linear_address_bits <= MAX_VIRTUAL_ADDRESS_LOG2);

	const size_t mask = ((uintptr_t)1 << zerOS_boot_cpu_linear_address_bits) - 1;
	return addr & mask;
}

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_virtaddr_sign_extend(uintptr_t addr)
{
	extern size_t zerOS_boot_cpu_linear_address_bits;
	assert(
	  zerOS_boot_cpu_linear_address_bits >= 32
	  && zerOS_boot_cpu_linear_address_bits <= MAX_VIRTUAL_ADDRESS_LOG2);

	const size_t shift = UINTPTR_WIDTH - zerOS_boot_cpu_linear_address_bits;
	return (uintptr_t)((intptr_t)(addr << shift) >> shift);
}

[[__gnu__::__always_inline__]] [[__gnu__::__const__]]
static inline uintptr_t zerOS_virtaddr_canonicalize(uintptr_t addr)
{
	return zerOS_virtaddr_sign_extend(addr);
}

#endif
