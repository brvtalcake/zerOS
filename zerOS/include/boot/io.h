#ifndef zerOS_BOOT_IO_H_INCLUDED
#define zerOS_BOOT_IO_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

#include <misc/sections.h>

BOOT_FUNC
extern uint8_t zerOS_inb(uint16_t port);
BOOT_FUNC
extern uint16_t zerOS_inw(uint16_t port);
BOOT_FUNC
extern uint32_t zerOS_inl(uint16_t port);

BOOT_FUNC
extern void zerOS_outb(uint16_t port, uint8_t val);
BOOT_FUNC
extern void zerOS_outw(uint16_t port, uint16_t val);
BOOT_FUNC
extern void zerOS_outl(uint16_t port, uint32_t val);

BOOT_FUNC
extern uint64_t zerOS_read_msr(uint32_t msr);
BOOT_FUNC
extern void zerOS_write_msr(uint32_t msr, uint64_t val);

#endif
