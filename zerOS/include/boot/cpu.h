#ifndef zerOS_BOOT_CPU_H_INCLUDED
#define zerOS_BOOT_CPU_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#include <boot/io.h>
#include <boot/misc.h>

#include <misc/sections.h>

#include <klibc/detail/enum.h>

enum zerOS_supported_cpus
{
    zerOS_CPU_X86_64_alderlake = 0,
    // TODO: Add support for more Intel and AMD CPUs
    zerOS_SUPPORTED_CPU_COUNT  = 1
};

struct zerOS_cpuid_info
{
    uint32_t eax;
    uint32_t ebx;
    uint32_t ecx;
    uint32_t edx;
};

enum zerOS_intel_cpu_feature_ecx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_ECX_CPU_FEATURE_SSE3         = (UINT32_C(1) << 0),
    zerOS_ECX_CPU_FEATURE_PCLMULQDQ    = (UINT32_C(1) << 1),
    zerOS_ECX_CPU_FEATURE_DTES64       = (UINT32_C(1) << 2),
    zerOS_ECX_CPU_FEATURE_MONITOR      = (UINT32_C(1) << 3),
    zerOS_ECX_CPU_FEATURE_DS_CPL       = (UINT32_C(1) << 4),
    zerOS_ECX_CPU_FEATURE_VMX          = (UINT32_C(1) << 5),
    zerOS_ECX_CPU_FEATURE_SMX          = (UINT32_C(1) << 6),
    zerOS_ECX_CPU_FEATURE_EIST         = (UINT32_C(1) << 7),
    zerOS_ECX_CPU_FEATURE_TM2          = (UINT32_C(1) << 8),
    zerOS_ECX_CPU_FEATURE_SSSE3        = (UINT32_C(1) << 9),
    zerOS_ECX_CPU_FEATURE_CNXT_ID      = (UINT32_C(1) << 10),
    zerOS_ECX_CPU_FEATURE_SDBG         = (UINT32_C(1) << 11),
    zerOS_ECX_CPU_FEATURE_FMA          = (UINT32_C(1) << 12),
    zerOS_ECX_CPU_FEATURE_CMPXCHG16B   = (UINT32_C(1) << 13),
    zerOS_ECX_CPU_FEATURE_XTPR_UPDCTL  = (UINT32_C(1) << 14),
    zerOS_ECX_CPU_FEATURE_PDCM         = (UINT32_C(1) << 15),
    // RESERVED                        = (UINT32_C(1) << 16),
    zerOS_ECX_CPU_FEATURE_PCID         = (UINT32_C(1) << 17),
    zerOS_ECX_CPU_FEATURE_DCA          = (UINT32_C(1) << 18),
    zerOS_ECX_CPU_FEATURE_SSE4_1       = (UINT32_C(1) << 19),
    zerOS_ECX_CPU_FEATURE_SSE4_2       = (UINT32_C(1) << 20),
    zerOS_ECX_CPU_FEATURE_X2APIC       = (UINT32_C(1) << 21),
    zerOS_ECX_CPU_FEATURE_MOVBE        = (UINT32_C(1) << 22),
    zerOS_ECX_CPU_FEATURE_POPCNT       = (UINT32_C(1) << 23),
    zerOS_ECX_CPU_FEATURE_TSC_DEADLINE = (UINT32_C(1) << 24),
    zerOS_ECX_CPU_FEATURE_AESNI        = (UINT32_C(1) << 25),
    zerOS_ECX_CPU_FEATURE_XSAVE        = (UINT32_C(1) << 26),
    zerOS_ECX_CPU_FEATURE_OSXSAVE      = (UINT32_C(1) << 27),
    zerOS_ECX_CPU_FEATURE_AVX          = (UINT32_C(1) << 28),
    zerOS_ECX_CPU_FEATURE_F16C         = (UINT32_C(1) << 29),
    zerOS_ECX_CPU_FEATURE_RDRAND       = (UINT32_C(1) << 30)
    // NOT USED                        = (UINT32_C(1) << 31)
};

enum zerOS_intel_cpu_feature_edx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_EDX_CPU_FEATURE_FPU          = (UINT32_C(1) << 0),
    zerOS_EDX_CPU_FEATURE_VME          = (UINT32_C(1) << 1),
    zerOS_EDX_CPU_FEATURE_DE           = (UINT32_C(1) << 2),
    zerOS_EDX_CPU_FEATURE_PSE          = (UINT32_C(1) << 3),
    zerOS_EDX_CPU_FEATURE_TSC          = (UINT32_C(1) << 4),
    zerOS_EDX_CPU_FEATURE_MSR          = (UINT32_C(1) << 5),
    zerOS_EDX_CPU_FEATURE_PAE          = (UINT32_C(1) << 6),
    zerOS_EDX_CPU_FEATURE_MCE          = (UINT32_C(1) << 7),
    zerOS_EDX_CPU_FEATURE_CX8          = (UINT32_C(1) << 8),
    zerOS_EDX_CPU_FEATURE_APIC         = (UINT32_C(1) << 9),
    // RESERVED                        = (UINT32_C(1) << 10),
    zerOS_EDX_CPU_FEATURE_SEP          = (UINT32_C(1) << 11),
    zerOS_EDX_CPU_FEATURE_MTRR         = (UINT32_C(1) << 12),
    zerOS_EDX_CPU_FEATURE_PGE          = (UINT32_C(1) << 13),
    zerOS_EDX_CPU_FEATURE_MCA          = (UINT32_C(1) << 14),
    zerOS_EDX_CPU_FEATURE_CMOV         = (UINT32_C(1) << 15),
    zerOS_EDX_CPU_FEATURE_PAT          = (UINT32_C(1) << 16),
    zerOS_EDX_CPU_FEATURE_PSE36        = (UINT32_C(1) << 17),
    zerOS_EDX_CPU_FEATURE_PSN          = (UINT32_C(1) << 18),
    zerOS_EDX_CPU_FEATURE_CLFSH        = (UINT32_C(1) << 19),
    // RESERVED                        = (UINT32_C(1) << 20),
    zerOS_EDX_CPU_FEATURE_DS           = (UINT32_C(1) << 21),
    zerOS_EDX_CPU_FEATURE_ACPI         = (UINT32_C(1) << 22),
    zerOS_EDX_CPU_FEATURE_MMX          = (UINT32_C(1) << 23),
    zerOS_EDX_CPU_FEATURE_FXSR         = (UINT32_C(1) << 24),
    zerOS_EDX_CPU_FEATURE_SSE          = (UINT32_C(1) << 25),
    zerOS_EDX_CPU_FEATURE_SSE2         = (UINT32_C(1) << 26),
    zerOS_EDX_CPU_FEATURE_SS           = (UINT32_C(1) << 27),
    zerOS_EDX_CPU_FEATURE_HTT          = (UINT32_C(1) << 28),
    zerOS_EDX_CPU_FEATURE_TM           = (UINT32_C(1) << 29),
    // RESERVED                        = (UINT32_C(1) << 30),
    zerOS_EDX_CPU_FEATURE_PBE          = (UINT32_C(1) << 31)
};

enum zerOS_intel_strext_cpu_feature_leaf0_ebx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_EBX_STREXT_CPU_FEATURE_FSGSBASE   = (UINT32_C(1) << 0),
    zerOS_EBX_STREXT_CPU_FEATURE_TSC_ADJUST = (UINT32_C(1) << 1),
    zerOS_EBX_STREXT_CPU_FEATURE_SGX        = (UINT32_C(1) << 2),
    
}

/**
 * @enum zerOS_intel_msr_address
 * @brief An enumeration of Intel MSR addresses.
 * @details This enumeration is used to define the addresses of Intel IA32 MSRs.
 */
enum zerOS_intel_msr_address
    UNDERLYING_TYPE(uint32_t)
{

};

static constexpr const uint32_t zerOS_intel_cpuid_valid_eaxs[] = {
    // Basic CPUID information
    // 0x0 - 0x2
    0x0, // Maximum CPUID leaf
    0x1, // Processor info, feature bits, and various other information
    0x2, // Cache and TLB information

    // CPUID information only visible when IA32_MISC_ENABLE[bit 22] \
       is set to default value 0
    // 0x3 - 0x7, 0x9 - 0xB, 0xD - 0x10, 0x12, 0x14 - 0x20
    0x3,  // Processor serial number if available
    0x4,  // Deterministic cache parameters for each level
    0x5,  // MONITOR/MWAIT parameters
    0x6,  // Thermal and power management parameters
    0x7,  // Structured Extended Feature Flags Enumeration
    0x9,  // Direct Cache Access information
    0xA,  // Architectural Performance Monitoring
    0xB,  // Extended Topology Enumeration
    0xD,  // Processor extended state enumeration
    0xF,  // Intel RDT monitoring enumeration + L3 Cache Intel RDT Monitoring Capability Enumeration
    0x10, // Intel RDT allocation enumeration + [L3/L2 Cache / Memory Bandwidth] Allocation Technology Enumeration
    0x12, // Intel SGX-related enumeration
    0x14, // Processor Trace Enumeration
    0x15, // Time Stamp Counter and Core Crystal Clock Information
    0x16, // Processor Frequency Information
    0x17, // System-On-Chip Vendor Attribute Enumeration
    0x18, // Deterministic Address Translation Parameters
    0x19, // Key Locker
    0x1A, // Native Model ID Enumeration
    0x1B, // PCONFIG Information
    0x1C, // Last Branch Records Information
    0x1D, // Tile Information
    0x1E, // TMUL Information
    0x1F, // V2 Extended Topology Enumeration
    0x20, // Processor History Reset

    // Extended CPUID information
    // 0x80000000 - 0x80000004, 0x80000006 - 0x80000008
    0x80000000, // Maximum extended CPUID leaf
    0x80000001, // Extended processor info and feature bits
    0x80000002, // Processor brand string
    0x80000003, // Processor brand string continued
    0x80000004, // Processor brand string continued
    0x80000006, // Extended L2 cache features
    0x80000007, // Invariant TSC available if bit 8 is set
    0x80000008  // Address range and physical address size
};

static constexpr const uint32_t zerOS_amd_cpuid_valid_eaxs[] = {
    // TODO
};

BOOT_FUNC
extern bool zerOS_cpuid_explicit(uint32_t leaf, uint32_t subleaf, struct zerOS_cpuid_info* info);
BOOT_FUNC
extern bool zerOS_cpuid(uint32_t leaf, struct zerOS_cpuid_info* info);

BOOT_FUNC
extern void zerOS_set_ia32_misc(bool value, uint8_t bit);

#endif
