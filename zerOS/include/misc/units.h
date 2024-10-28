#ifndef zerOS_MISC_UNITS_H_INCLUDED
#define zerOS_MISC_UNITS_H_INCLUDED

#include <stdint.h>

#undef  TiB
#define TiB (UINT64_C(1024) * UINT64_C(1024) * UINT64_C(1024) * UINT64_C(1024))

#undef  GiB
#define GiB (UINT64_C(1024) * UINT64_C(1024) * UINT64_C(1024))

#undef  MiB
#define MiB (UINT64_C(1024) * UINT64_C(1024))

#undef  KiB
#define KiB (UINT64_C(1024))

#undef  TB
#define TB (UINT64_C(1000) * UINT64_C(1000) * UINT64_C(1000) * UINT64_C(1000))

#undef  GB
#define GB (UINT64_C(1000) * UINT64_C(1000) * UINT64_C(1000))

#undef  MB
#define MB (UINT64_C(1000) * UINT64_C(1000))

#undef  KB
#define KB (UINT64_C(1000))

#endif
