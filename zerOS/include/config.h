 
#ifndef zerOS_CONFIG_H_INCLUDED
#define zerOS_CONFIG_H_INCLUDED

#undef  zerOS_CONFIG_CPU
#define zerOS_CONFIG_CPU alderlake

#undef  zerOS_CONFIG_ARCH
#define zerOS_CONFIG_ARCH x86_64

#undef  zerOS_CONFIG_DEBUG
#define zerOS_CONFIG_DEBUG false

#undef  zerOS_CONFIG_CPU_FEATURES
#define zerOS_CONFIG_CPU_FEATURES  fpu vme de pse tsc msr pae mce cx8 apic mtrr sep pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe pni pclmul dts64 monitor ds_cpl vmx est tm2 ssse3 cx16 xtpr pdcm sse4_1 sse4_2 syscall xd movbe popcnt aes xsave osxsave avx rdtscp lm lahf_lm abm constant_tsc fma3 f16c rdrand x2apic avx2 bmi1 bmi2 sha_ni rdseed adx

#undef  zerOS_CONFIG_UNDER_QEMU
#define zerOS_CONFIG_UNDER_QEMU 1

#undef  zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS
#define zerOS_CONFIG_MAX_USABLE_MEMORY_REGIONS 64UL






#undef  zerOS_CONFIG_HAVE_128BITS_LONGLONG
#define zerOS_CONFIG_HAVE_128BITS_LONGLONG 0

#undef  zerOS_CONFIG_HAVE_128BITS_LONGDOUBLE
#define zerOS_CONFIG_HAVE_128BITS_LONGDOUBLE 1

#undef  zerOS_CONFIG_HAVE_VA_OPT
#define zerOS_CONFIG_HAVE_VA_OPT 1
#endif
