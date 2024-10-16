#include <stdarg.h>

#include <kernel/serial/ports.h>
#include <kernel/compiler/cast.h>

#include <misc/sections.h>
#include <misc/printf.h>

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
static void write_debugcon(char a)
{
    zerOS_outb(0xe9, a);
}

BOOT_FUNC
static void uint_to_string(unsigned int value, char* buffer, int base)
{
}


BOOT_FUNC VPRINTF_LIKE(1)
/**
 * @fn int zerOS_early_vprintk(const char* str, va_list varargs)
 * @brief Writes a formatted string to the early consoles.
 * @details Here are the supported format specifiers:
 *    - `%d`: Prints a signed integer.
 *    - `%u`: Prints an unsigned integer.
 *    - `%x`: Prints an unsigned integer in hexadecimal (lowercase).
 *    - `%X`: Prints an unsigned integer in hexadecimal (uppercase).
 *    - `%p`: Prints a pointer.
 *    - `%b`: Prints an unsigned integer in binary.
 *    - `%o`: Prints an unsigned integer in octal.
 *    - `%f`: Prints a floating-point number.
 *    - `%c`: Prints a character.
 *    - `%s`: Prints a null-terminated string.
 *    - `%n`: Prints the number of characters written so far.
 *    - `%%`: Prints a percent sign.
 * @param str The format string.
 * @param varargs The format arguments.
 * @return The number of characters written.
 */
extern int zerOS_early_vprintk(const char* str, va_list varargs)
{
    int written = 0;

    for (const char* ptr = str; *ptr != '\0'; ptr++)
    {
        if (*ptr == '%')
        {
            ptr++;

            switch (*ptr)
            {
                case 'd': {
                    int value = va_arg(varargs, int);
                    
                } break;
            }
        }
        else
        {
            write_serial(zerOS_SERIAL_DEBUG, *ptr);
            write_debugcon(*ptr);
            written++;
        }
    }

    return written;
}

BOOT_FUNC PRINTF_LIKE(1, 2)
/**
 * @fn int zerOS_early_printk(const char* str, ...)
 * @brief Writes a formatted string to the early consoles.
 * @param str The format string.
 * @param ... The format arguments.
 * @return The number of characters written.
 * @note This function features the same format string syntax as `zerOS_early_vprintk`.
 */
extern int zerOS_early_printk(const char* str, ...)
{
    int written = 0;
    va_list args;

    va_start(args, str);
    written = zerOS_early_vprintk(str, args);
    va_end(args);

    return written;
}