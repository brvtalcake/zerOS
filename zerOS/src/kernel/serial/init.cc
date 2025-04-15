
#include <kernel/cpu/io.h>
#include <kernel/serial/ports.h>

#include <misc/sections.h>

BOOT_FUNC
static inline bool serial_init(enum zerOS_serial_port port)
{
    inline bool test_is_faulty(void)
    {
        zerOS_outb(port + 0, 0xae);
        return zerOS_inb(port + 0) != 0xae;
    };

    zerOS_outb(port + 1, 0x00); // Disable all interrupts
    zerOS_outb(port + 3, 0x80); // Enable DLAB (set baud rate divisor)
    zerOS_outb(port + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
    zerOS_outb(port + 1, 0x00); //                  (hi byte)
    zerOS_outb(port + 3, 0x03); // 8 bits, no parity, one stop bit
    zerOS_outb(port + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
    zerOS_outb(port + 4, 0x0B); // IRQs enabled, RTS/DSR set
    zerOS_outb(port + 4, 0x1E); // Set in loopback mode, test the serial chip

    if (test_is_faulty())
        return false;

    zerOS_outb(port + 4, 0x0F); // Set normal operation mode

    return true;
}

BOOT_FUNC
extern bool zerOS_serial_early_init(void)
{
    return serial_init(zerOS_SERIAL_DEBUG);
}