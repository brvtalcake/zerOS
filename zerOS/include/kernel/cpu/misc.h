#ifndef zerOS_KERNEL_CPU_MISC_H_INCLUDED
#define zerOS_KERNEL_CPU_MISC_H_INCLUDED

#include <misc/sections.h>

extern void zerOS_halt(void);
extern void zerOS_reboot(void);
extern void zerOS_cli(void);
extern void zerOS_hcf(void);

#endif
