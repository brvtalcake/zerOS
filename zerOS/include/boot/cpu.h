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
    zerOS_INTEL_ECX_CPU_FEATURE_SSE3         = (UINT32_C(1) << 0),
    zerOS_INTEL_ECX_CPU_FEATURE_PCLMULQDQ    = (UINT32_C(1) << 1),
    zerOS_INTEL_ECX_CPU_FEATURE_DTES64       = (UINT32_C(1) << 2),
    zerOS_INTEL_ECX_CPU_FEATURE_MONITOR      = (UINT32_C(1) << 3),
    zerOS_INTEL_ECX_CPU_FEATURE_DS_CPL       = (UINT32_C(1) << 4),
    zerOS_INTEL_ECX_CPU_FEATURE_VMX          = (UINT32_C(1) << 5),
    zerOS_INTEL_ECX_CPU_FEATURE_SMX          = (UINT32_C(1) << 6),
    zerOS_INTEL_ECX_CPU_FEATURE_EIST         = (UINT32_C(1) << 7),
    zerOS_INTEL_ECX_CPU_FEATURE_TM2          = (UINT32_C(1) << 8),
    zerOS_INTEL_ECX_CPU_FEATURE_SSSE3        = (UINT32_C(1) << 9),
    zerOS_INTEL_ECX_CPU_FEATURE_CNXT_ID      = (UINT32_C(1) << 10),
    zerOS_INTEL_ECX_CPU_FEATURE_SDBG         = (UINT32_C(1) << 11),
    zerOS_INTEL_ECX_CPU_FEATURE_FMA          = (UINT32_C(1) << 12),
    zerOS_INTEL_ECX_CPU_FEATURE_CMPXCHG16B   = (UINT32_C(1) << 13),
    zerOS_INTEL_ECX_CPU_FEATURE_XTPR_UPDCTL  = (UINT32_C(1) << 14),
    zerOS_INTEL_ECX_CPU_FEATURE_PDCM         = (UINT32_C(1) << 15),
    // RESERVED                              = (UINT32_C(1) << 16),
    zerOS_INTEL_ECX_CPU_FEATURE_PCID         = (UINT32_C(1) << 17),
    zerOS_INTEL_ECX_CPU_FEATURE_DCA          = (UINT32_C(1) << 18),
    zerOS_INTEL_ECX_CPU_FEATURE_SSE4_1       = (UINT32_C(1) << 19),
    zerOS_INTEL_ECX_CPU_FEATURE_SSE4_2       = (UINT32_C(1) << 20),
    zerOS_INTEL_ECX_CPU_FEATURE_X2APIC       = (UINT32_C(1) << 21),
    zerOS_INTEL_ECX_CPU_FEATURE_MOVBE        = (UINT32_C(1) << 22),
    zerOS_INTEL_ECX_CPU_FEATURE_POPCNT       = (UINT32_C(1) << 23),
    zerOS_INTEL_ECX_CPU_FEATURE_TSC_DEADLINE = (UINT32_C(1) << 24),
    zerOS_INTEL_ECX_CPU_FEATURE_AESNI        = (UINT32_C(1) << 25),
    zerOS_INTEL_ECX_CPU_FEATURE_XSAVE        = (UINT32_C(1) << 26),
    zerOS_INTEL_ECX_CPU_FEATURE_OSXSAVE      = (UINT32_C(1) << 27),
    zerOS_INTEL_ECX_CPU_FEATURE_AVX          = (UINT32_C(1) << 28),
    zerOS_INTEL_ECX_CPU_FEATURE_F16C         = (UINT32_C(1) << 29),
    zerOS_INTEL_ECX_CPU_FEATURE_RDRAND       = (UINT32_C(1) << 30)
    // NOT USED                              = (UINT32_C(1) << 31)
};

enum zerOS_intel_cpu_feature_edx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_INTEL_EDX_CPU_FEATURE_FPU          = (UINT32_C(1) << 0),
    zerOS_INTEL_EDX_CPU_FEATURE_VME          = (UINT32_C(1) << 1),
    zerOS_INTEL_EDX_CPU_FEATURE_DE           = (UINT32_C(1) << 2),
    zerOS_INTEL_EDX_CPU_FEATURE_PSE          = (UINT32_C(1) << 3),
    zerOS_INTEL_EDX_CPU_FEATURE_TSC          = (UINT32_C(1) << 4),
    zerOS_INTEL_EDX_CPU_FEATURE_MSR          = (UINT32_C(1) << 5),
    zerOS_INTEL_EDX_CPU_FEATURE_PAE          = (UINT32_C(1) << 6),
    zerOS_INTEL_EDX_CPU_FEATURE_MCE          = (UINT32_C(1) << 7),
    zerOS_INTEL_EDX_CPU_FEATURE_CX8          = (UINT32_C(1) << 8),
    zerOS_INTEL_EDX_CPU_FEATURE_APIC         = (UINT32_C(1) << 9),
    // RESERVED                              = (UINT32_C(1) << 10),
    zerOS_INTEL_EDX_CPU_FEATURE_SEP          = (UINT32_C(1) << 11),
    zerOS_INTEL_EDX_CPU_FEATURE_MTRR         = (UINT32_C(1) << 12),
    zerOS_INTEL_EDX_CPU_FEATURE_PGE          = (UINT32_C(1) << 13),
    zerOS_INTEL_EDX_CPU_FEATURE_MCA          = (UINT32_C(1) << 14),
    zerOS_INTEL_EDX_CPU_FEATURE_CMOV         = (UINT32_C(1) << 15),
    zerOS_INTEL_EDX_CPU_FEATURE_PAT          = (UINT32_C(1) << 16),
    zerOS_INTEL_EDX_CPU_FEATURE_PSE36        = (UINT32_C(1) << 17),
    zerOS_INTEL_EDX_CPU_FEATURE_PSN          = (UINT32_C(1) << 18),
    zerOS_INTEL_EDX_CPU_FEATURE_CLFSH        = (UINT32_C(1) << 19),
    // RESERVED                              = (UINT32_C(1) << 20),
    zerOS_INTEL_EDX_CPU_FEATURE_DS           = (UINT32_C(1) << 21),
    zerOS_INTEL_EDX_CPU_FEATURE_ACPI         = (UINT32_C(1) << 22),
    zerOS_INTEL_EDX_CPU_FEATURE_MMX          = (UINT32_C(1) << 23),
    zerOS_INTEL_EDX_CPU_FEATURE_FXSR         = (UINT32_C(1) << 24),
    zerOS_INTEL_EDX_CPU_FEATURE_SSE          = (UINT32_C(1) << 25),
    zerOS_INTEL_EDX_CPU_FEATURE_SSE2         = (UINT32_C(1) << 26),
    zerOS_INTEL_EDX_CPU_FEATURE_SS           = (UINT32_C(1) << 27),
    zerOS_INTEL_EDX_CPU_FEATURE_HTT          = (UINT32_C(1) << 28),
    zerOS_INTEL_EDX_CPU_FEATURE_TM           = (UINT32_C(1) << 29),
    // RESERVED                              = (UINT32_C(1) << 30),
    zerOS_INTEL_EDX_CPU_FEATURE_PBE          = (UINT32_C(1) << 31)
};

enum zerOS_intel_strext_cpu_feature_subleaf0_ebx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_FSGSBASE    = (UINT32_C(1) << 0),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_TSC_ADJUST  = (UINT32_C(1) << 1),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_SGX         = (UINT32_C(1) << 2),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_BMI1        = (UINT32_C(1) << 3),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_HLE         = (UINT32_C(1) << 4),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX2        = (UINT32_C(1) << 5),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_FDP_EXCPTN  = (UINT32_C(1) << 6),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_SMEP        = (UINT32_C(1) << 7),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_BMI2        = (UINT32_C(1) << 8),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_ERMS        = (UINT32_C(1) << 9),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_INVPCID     = (UINT32_C(1) << 10),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_RTM         = (UINT32_C(1) << 11),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_RDT_M       = (UINT32_C(1) << 12),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_DEP_CS_DS   = (UINT32_C(1) << 13),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_MPX         = (UINT32_C(1) << 14),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_RDT_A       = (UINT32_C(1) << 15),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512F     = (UINT32_C(1) << 16),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512DQ    = (UINT32_C(1) << 17),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_RDSEED      = (UINT32_C(1) << 18),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_ADX         = (UINT32_C(1) << 19),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_SMAP        = (UINT32_C(1) << 20),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512_IFMA = (UINT32_C(1) << 21),
    // RESERVED                                    = (UINT32_C(1) << 22),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_CLFLUSHOPT  = (UINT32_C(1) << 23),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_CLWB        = (UINT32_C(1) << 24),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_INTEL_PT    = (UINT32_C(1) << 25),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512PF    = (UINT32_C(1) << 26), // (Xeon Phi only)
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512ER    = (UINT32_C(1) << 27), // (Xeon Phi only)
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512CD    = (UINT32_C(1) << 28),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_SHA         = (UINT32_C(1) << 29),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512BW    = (UINT32_C(1) << 30),
    zerOS_INTEL_EBX_STREXT_CPU_FEATURE_AVX512VL    = (UINT32_C(1) << 31)
};

enum zerOS_intel_strext_cpu_feature_subleaf0_ecx_bits
    UNDERLYING_TYPE(uint32_t)
{
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_PREFETCHWT1      = (UINT32_C(1) << 0), // (Xeon Phi only)
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_AVX512_VBMI      = (UINT32_C(1) << 1),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_UMIP             = (UINT32_C(1) << 2),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_PKU              = (UINT32_C(1) << 3),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_OSPKE            = (UINT32_C(1) << 4),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_WAITPKG          = (UINT32_C(1) << 5),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_AVX512_VBMI2     = (UINT32_C(1) << 6),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_CET_SS           = (UINT32_C(1) << 7),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_GFNI             = (UINT32_C(1) << 8),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_VAES             = (UINT32_C(1) << 9),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_VPCLMULQDQ       = (UINT32_C(1) << 10),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_AVX512_VNNI      = (UINT32_C(1) << 11),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_AVX512_BITALG    = (UINT32_C(1) << 12),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_TME_EN           = (UINT32_C(1) << 13),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_AVX512_VPOPCNTDQ = (UINT32_C(1) << 14),
    // RESERVED                                         = (UINT32_C(1) << 15),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_LA57             = (UINT32_C(1) << 16),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_MAWAU            = (UINT32_C(1) << 17)
                                                        | (UINT32_C(1) << 18)
                                                        | (UINT32_C(1) << 19)
                                                        | (UINT32_C(1) << 20)
                                                        | (UINT32_C(1) << 21),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_RDPID            = (UINT32_C(1) << 22),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_KEYLOCKER        = (UINT32_C(1) << 23),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_BUS_LOCK_DETECT  = (UINT32_C(1) << 24),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_CLDEMOTE         = (UINT32_C(1) << 25),
    // RESERVED                                         = (UINT32_C(1) << 26),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_MOVDIRI          = (UINT32_C(1) << 27),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_MOVDIR64B        = (UINT32_C(1) << 28),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_ENQCMD           = (UINT32_C(1) << 29),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_SGX_LC           = (UINT32_C(1) << 30),
    zerOS_INTEL_ECX_STREXT_CPU_FEATURE_PKS              = (UINT32_C(1) << 31)
};

enum zerOS_intel_strext_cpu_feature_subleaf0_edx_bits
    UNDERLYING_TYPE(uint32_t)
{
    // RESERVED                                            = (UINT32_C(1) << 0),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_SGX_KEYS            = (UINT32_C(1) << 1),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AVX512_4VNNIW       = (UINT32_C(1) << 2), // (Xeon Phi only)
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AVX512_4FMAPS       = (UINT32_C(1) << 3), // (Xeon Phi only)
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_FSRM                = (UINT32_C(1) << 4),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_UINTR               = (UINT32_C(1) << 5),
    // RESERVED                                            = (UINT32_C(1) << 6),
    // RESERVED                                            = (UINT32_C(1) << 7),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AVX512_VP2INTERSECT = (UINT32_C(1) << 8),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_SRBDS_CTRL          = (UINT32_C(1) << 9),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_MD_CLEAR            = (UINT32_C(1) << 10),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_RTM_ALWAYS_ABORT    = (UINT32_C(1) << 11),
    // RESERVED                                            = (UINT32_C(1) << 12),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_RTM_FORCE_ABORT     = (UINT32_C(1) << 13),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_SERIALIZE           = (UINT32_C(1) << 14),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_HYBRID              = (UINT32_C(1) << 15),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_TSXLDTRK            = (UINT32_C(1) << 16),
    // RESERVED                                            = (UINT32_C(1) << 17),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_PCONFIG             = (UINT32_C(1) << 18),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_ARCH_LBR            = (UINT32_C(1) << 19),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_CET_IBT             = (UINT32_C(1) << 20),
    // RESERVED                                            = (UINT32_C(1) << 21),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AMX_BF16            = (UINT32_C(1) << 22),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AVX512_FP16         = (UINT32_C(1) << 23),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AMX_TILE            = (UINT32_C(1) << 24),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_AMX_INT8            = (UINT32_C(1) << 25),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_IBRS_IBPB           = (UINT32_C(1) << 26),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_STIBP               = (UINT32_C(1) << 27),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_L1D_FLUSH           = (UINT32_C(1) << 28),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_IA32_ARCH_CAP       = (UINT32_C(1) << 29),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_IA32_CORE_CAP       = (UINT32_C(1) << 30),
    zerOS_INTEL_EDX_STREXT_CPU_FEATURE_SSBD                = (UINT32_C(1) << 31)
};

enum zerOS_intel_strext_cpu_feature_subleaf1_eax_bits
    UNDERLYING_TYPE(uint32_t)
{
    // RESERVED                                            = (UINT32_C(1) << 0),
    // RESERVED                                            = (UINT32_C(1) << 1),
    // RESERVED                                            = (UINT32_C(1) << 2),
    // RESERVED                                            = (UINT32_C(1) << 3),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_AVX_VNNI            = (UINT32_C(1) << 4),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_AVX512_BF16         = (UINT32_C(1) << 5),
    // RESERVED                                            = (UINT32_C(1) << 6),
    // RESERVED                                            = (UINT32_C(1) << 7),
    // RESERVED                                            = (UINT32_C(1) << 8),
    // RESERVED                                            = (UINT32_C(1) << 9),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_FAST_0LEN_REPMOVSB  = (UINT32_C(1) << 10),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_FAST_SHORT_REPSTOSB = (UINT32_C(1) << 11),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_FAST_SHORT_REPCMPSB = (UINT32_C(1) << 12),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_FAST_SHORT_REPSCASB = (UINT32_C(1) << 12),
    // RESERVED                                            = (UINT32_C(1) << 13),
    // RESERVED                                            = (UINT32_C(1) << 14),
    // RESERVED                                            = (UINT32_C(1) << 15),
    // RESERVED                                            = (UINT32_C(1) << 16),
    // RESERVED                                            = (UINT32_C(1) << 17),
    // RESERVED                                            = (UINT32_C(1) << 18),
    // RESERVED                                            = (UINT32_C(1) << 19),
    // RESERVED                                            = (UINT32_C(1) << 20),
    // RESERVED                                            = (UINT32_C(1) << 21),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_HRESET              = (UINT32_C(1) << 22),
    // RESERVED                                            = (UINT32_C(1) << 23),
    // RESERVED                                            = (UINT32_C(1) << 24),
    // RESERVED                                            = (UINT32_C(1) << 25),
    // RESERVED                                            = (UINT32_C(1) << 26),
    // RESERVED                                            = (UINT32_C(1) << 27),
    // RESERVED                                            = (UINT32_C(1) << 28),
    // RESERVED                                            = (UINT32_C(1) << 29),
    zerOS_INTEL_EAX_STREXT_CPU_FEATURE_NO_INVD_POST_BIOS   = (UINT32_C(1) << 30),
    // RESERVED                                            = (UINT32_C(1) << 31)
};

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
extern bool zerOS_cpuid_count(uint32_t leaf, uint32_t subleaf, struct zerOS_cpuid_info* info);
BOOT_FUNC
extern bool zerOS_cpuid(uint32_t leaf, struct zerOS_cpuid_info* info);

BOOT_FUNC
extern void zerOS_set_ia32_misc(bool value, uint8_t bit);

#endif
