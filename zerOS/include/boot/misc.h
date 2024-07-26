#ifndef zerOS_BOOT_MISC_H_INCLUDED
#define zerOS_BOOT_MISC_H_INCLUDED

#include <misc/sections.h>

BOOT_FUNC
extern void zerOS_halt(void);

BOOT_FUNC
extern void zerOS_reboot(void);

BOOT_FUNC
extern void zerOS_cli(void);

BOOT_FUNC
extern void zerOS_hcf(void);

#endif
