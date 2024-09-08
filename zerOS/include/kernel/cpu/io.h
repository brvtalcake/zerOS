#ifndef zerOS_KERNEL_CPU_IO_H_INCLUDED
#define zerOS_KERNEL_CPU_IO_H_INCLUDED

#include <stddef.h>
#include <stdint.h>

extern uint8_t zerOS_inb(uint16_t port);
extern uint16_t zerOS_inw(uint16_t port);
extern uint32_t zerOS_inl(uint16_t port);

extern void zerOS_outb(uint16_t port, uint8_t val);
extern void zerOS_outw(uint16_t port, uint16_t val);
extern void zerOS_outl(uint16_t port, uint32_t val);

extern uint64_t zerOS_read_msr(uint32_t msr);
extern void zerOS_write_msr(uint32_t msr, uint64_t val);

#endif
