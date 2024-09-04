#ifndef zerOS_MACHINE_ALDERLAKE_MSR_H_INCLUDED
#define zerOS_MACHINE_ALDERLAKE_MSR_H_INCLUDED

#include <stddef.h>
#include <stdint.h>
#include <klibc/detail/enum.h>

enum zerOS_msr_address
    UNDERLYING_TYPE(uintmax_t)
{
    zerOS_MSR_COUNT = 0
};

#endif
