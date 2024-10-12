#include <stdarg.h>

#include <kernel/serial/ports.h>

#include <misc/sections.h>

BOOT_FUNC
static bool is_transmit_empty(enum zerOS_serial_port port)
{
    return zerOS_inb(port + 5) & 0x20;
}

BOOT_FUNC
static void write_serial(enum zerOS_serial_port port, char a)
{
    while (is_transmit_empty(port) == 0);

    zerOS_outb(port, a);
}

BOOT_FUNC
extern int zerOS_early_printk(const char* str, ...)
{
    va_list args;
    va_start(args, str);

    for (size_t i = 0; str[i] != '\0'; i++)
    {
        if (str[i] == '%')
        {
            i++;
            switch (str[i])
            {
                case 'c':
                {
                    char c = va_arg(args, char);
                    write_serial(zerOS_SERIAL_DEBUG, c);
                    break;
                }
                case 's':
                {
                    const char* s = va_arg(args, const char*);
                    for (size_t j = 0; s[j] != '\0'; j++)
                        write_serial(zerOS_SERIAL_DEBUG, s[j]);
                    break;
                }
                case 'd':
                {
                    int d = va_arg(args, int);
                    if (d < 0)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, '-');
                        d = -d;
                    }
                    char buf[32];
                    size_t j = 0;
                    do
                    {
                        buf[j++] = (d % 10) + '0';
                        d /= 10;
                    } while (d != 0);
                    for (size_t k = j; k > 0; k--)
                        write_serial(zerOS_SERIAL_DEBUG, buf[k - 1]);
                    break;
                }
                case 'x':
                {
                    unsigned int x = va_arg(args, unsigned int);
                    char buf[32];
                    size_t j = 0;
                    do
                    {
                        unsigned int digit = x % 16;
                        buf[j++] = digit + (digit < 10 ? '0' : 'a' - 10);
                        x /= 16;
                    } while (x != 0);
                    for (size_t k = j; k > 0; k--)
                        write_serial(zerOS_SERIAL_DEBUG, buf[k - 1]);
                    break;
                }
                case '%':
                    write_serial(zerOS_SERIAL_DEBUG, '%');
                    break;
                default:
                    write_serial(zerOS_SERIAL_DEBUG, str[i]);
                    break;
            }
        }
        else
        {
            write_serial(zerOS_SERIAL_DEBUG, str[i]);
        }
    }

    va_end(args);

    return 0;
}