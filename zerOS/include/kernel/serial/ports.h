#ifndef zerOS_KERNEL_SERIAL_PORTS_H_INCLUDED
#define zerOS_KERNEL_SERIAL_PORTS_H_INCLUDED

#include <kernel/compiler/enum.h>

enum zerOS_serial_port
    UNDERLYING_TYPE(uint16_t)
{
    zerOS_SERIAL_COM1 = 0x3f8,
    zerOS_SERIAL_COM2 = 0x2f8,
    zerOS_SERIAL_COM3 = 0x3e8,
    zerOS_SERIAL_COM4 = 0x2e8,

    zerOS_SERIAL_DEBUG = zerOS_SERIAL_COM1
};

BOOT_FUNC
extern bool zerOS_serial_early_init(void);

#endif
