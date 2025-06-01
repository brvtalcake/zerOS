/* !!! NO INCLUDE GUARDS DESIRED !!! */

#undef zerOS_PLATFORM_IS_X86
#undef zerOS_PLATFORM_IS_AMD64
#undef zerOS_PLATFORM_IS_ARM32
#undef zerOS_PLATFORM_IS_AARCH64
#undef zerOS_PLATFORM_IS_PPC32
#undef zerOS_PLATFORM_IS_PPC64
#undef zerOS_PLATFORM_IS_MIPS32
#undef zerOS_PLATFORM_IS_MIPS64
#undef zerOS_PLATFORM_IS_RISCV32
#undef zerOS_PLATFORM_IS_RISCV64
#undef zerOS_PLATFORM_IS_LOONGARCH64
#undef zerOS_PLATFORM_IS_SPARC32
#undef zerOS_PLATFORM_IS_SPARC64
#undef zerOS_PLATFORM_IS_AVR32
#undef zerOS_PLATFORM_IS_ZARCH

#if defined(__amd64__) || defined(__x86_64__)
	#define zerOS_PLATFORM_IS_AMD64 1
#else
	#define zerOS_PLATFORM_IS_AMD64 0
#endif

#if (                  \
  defined(__i386__)    \
  || defined(__i486__) \
  || defined(__i586__) \
  || defined(__i686__) \
  || defined(__x86__)) \
  && !zerOS_PLATFORM_IS_AMD64
	#define zerOS_PLATFORM_IS_X86 1
#else
	#define zerOS_PLATFORM_IS_X86 0
#endif

#if defined(__aarch64__)
	#define zerOS_PLATFORM_IS_AARCH64 1
#else
	#define zerOS_PLATFORM_IS_AARCH64 0
#endif

#if (defined(__arm__) || defined(__thumb__)) && !zerOS_PLATFORM_IS_AARCH64
	#define zerOS_PLATFORM_IS_ARM32 1
#else
	#define zerOS_PLATFORM_IS_ARM32 0
#endif

#if defined(__powerpc64__) || defined(__ppc64__) || defined(__PPC64__) || defined(_ARCH_PPC64)
	#define zerOS_PLATFORM_IS_PPC64 1
#else
	#define zerOS_PLATFORM_IS_PPC64 0
#endif

#if (                     \
  defined(__powerpc)      \
  || defined(__powerpc__) \
  || defined(__POWERPC__) \
  || defined(__ppc__)     \
  || defined(__PPC__)     \
  || defined(_ARCH_PPC))  \
  && !zerOS_PLATFORM_IS_PPC64
	#define zerOS_PLATFORM_IS_PPC32 1
#else
	#define zerOS_PLATFORM_IS_PPC32 0
#endif

#if defined(__mips__) && ((__mips == 64) || defined(__mips64__) || defined(__mips64))
	#define zerOS_PLATFORM_IS_MIPS64 1
#else
	#define zerOS_PLATFORM_IS_MIPS64 0
#endif

#if defined(__mips__) && (__mips == 32)
	#define zerOS_PLATFORM_IS_MIPS32 1
#else
	#define zerOS_PLATFORM_IS_MIPS32 0
#endif

#if defined(__riscv) && __riscv_xlen == 64
	#define zerOS_PLATFORM_IS_RISCV64 1
#else
	#define zerOS_PLATFORM_IS_RISCV64 0
#endif

#if defined(__riscv) && __riscv_xlen == 32
	#define zerOS_PLATFORM_IS_RISCV32 1
#else
	#define zerOS_PLATFORM_IS_RISCV32 0
#endif

#if defined(__loongarch__) && defined(__loongarch64)
	#define zerOS_PLATFORM_IS_LOONGARCH64 1
#else
	#define zerOS_PLATFORM_IS_LOONGARCH64 0
#endif

#if defined(__sparc__) && (!__sparc64__)
	#define zerOS_PLATFORM_IS_SPARC32 1
#else
	#define zerOS_PLATFORM_IS_SPARC32 0
#endif

#if defined(__sparc__) && defined(__sparc64__)
	#define zerOS_PLATFORM_IS_SPARC64 1
#else
	#define zerOS_PLATFORM_IS_SPARC64 0
#endif

#define zerOS_PLATFORM_IS_AVR32 0 /* TODO */

#if defined(__s390__) || defined(__s390x__) || defined(__zarch__)
	#define zerOS_PLATFORM_IS_ZARCH 1
#else
	#define zerOS_PLATFORM_IS_ZARCH 0
#endif

#if !defined(zerOS_PLATFORM_IS_X86)          \
  || !defined(zerOS_PLATFORM_IS_AMD64)       \
  || !defined(zerOS_PLATFORM_IS_ARM32)       \
  || !defined(zerOS_PLATFORM_IS_AARCH64)     \
  || !defined(zerOS_PLATFORM_IS_PPC32)       \
  || !defined(zerOS_PLATFORM_IS_PPC64)       \
  || !defined(zerOS_PLATFORM_IS_MIPS32)      \
  || !defined(zerOS_PLATFORM_IS_MIPS64)      \
  || !defined(zerOS_PLATFORM_IS_RISCV32)     \
  || !defined(zerOS_PLATFORM_IS_RISCV64)     \
  || !defined(zerOS_PLATFORM_IS_LOONGARCH64) \
  || !defined(zerOS_PLATFORM_IS_SPARC32)     \
  || !defined(zerOS_PLATFORM_IS_SPARC64)     \
  || !defined(zerOS_PLATFORM_IS_AVR32)       \
  || !defined(zerOS_PLATFORM_IS_ZARCH)
	#error "configuration error !"
#endif

static_assert(
  zerOS_PLATFORM_IS_X86
	  + zerOS_PLATFORM_IS_AMD64
	  + zerOS_PLATFORM_IS_ARM32
	  + zerOS_PLATFORM_IS_AARCH64
	  + zerOS_PLATFORM_IS_PPC32
	  + zerOS_PLATFORM_IS_PPC64
	  + zerOS_PLATFORM_IS_MIPS32
	  + zerOS_PLATFORM_IS_MIPS64
	  + zerOS_PLATFORM_IS_RISCV32
	  + zerOS_PLATFORM_IS_RISCV64
	  + zerOS_PLATFORM_IS_LOONGARCH64
	  + zerOS_PLATFORM_IS_SPARC32
	  + zerOS_PLATFORM_IS_SPARC64
	  + zerOS_PLATFORM_IS_AVR32
	  + zerOS_PLATFORM_IS_ZARCH
	== 1,
  "multiple platforms detected");

/* !!! NO INCLUDE GUARDS DESIRED !!! */
