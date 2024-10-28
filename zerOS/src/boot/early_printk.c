#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include <limits.h>

#include <kernel/qemu.h>
#include <kernel/serial/ports.h>
#include <kernel/compiler/cast.h>

#include <misc/sections.h>
#include <misc/printf.h>

#include <klibc/preprocessor/seq.h>

#include <chaos/preprocessor/recursion/basic.h>
#include <chaos/preprocessor/recursion/expr.h>
#include <chaos/preprocessor/punctuation.h>
#include <chaos/preprocessor/seq/size.h>
#include <chaos/preprocessor/seq/for_each_i.h>
#include <chaos/preprocessor/seq/for_each.h>
#include <chaos/preprocessor/stringize.h>
#include <chaos/preprocessor/cat.h>

#undef  __digit_chars_lower
#undef  __digit_chars_upper
#undef  __digit_chars_len
#define __digit_chars_lower (0)(1)(2)(3)(4)(5)(6)(7)(8)(9)(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)(m)(n)(o)(p)(q)(r)(s)(t)(u)(v)(w)(x)(y)(z)
#define __digit_chars_upper (0)(1)(2)(3)(4)(5)(6)(7)(8)(9)(A)(B)(C)(D)(E)(F)(G)(H)(I)(J)(K)(L)(M)(N)(O)(P)(Q)(R)(S)(T)(U)(V)(W)(X)(Y)(Z)
#define __digit_chars_len   36

#undef  __digits_tablelookup_build_impl_mapped
#define __digits_tablelookup_build_impl_mapped(_, sym1, sym2) CHAOS_PP_STRINGIZE(CHAOS_PP_CAT(sym1, sym2))

#undef  __digits_tablelookup_build_impl
#define __digits_tablelookup_build_impl(_, sym, syms)   \
    KLIBC_PP_SEQ_MAP(                                   \
        CHAOS_PP_EMPTY,                                 \
        __digits_tablelookup_build_impl_mapped,         \
        KLIBC_PP_SEQ_MAKE(                              \
            sym,                                        \
            __digit_chars_len                           \
        ),                                              \
        syms                                            \
    )


#undef  __digits_tablelookup_build
#define __digits_tablelookup_build(syms)        \
    CHAOS_PP_EXPR(                              \
        CHAOS_PP_SEQ_FOR_EACH(                  \
            __digits_tablelookup_build_impl,    \
            syms, syms                          \
        )                                       \
    )

#if 0
#ifndef __INTELLISENSE__
static const char lut_lower[] = __digits_tablelookup_build(__digit_chars_lower);
static const char lut_upper[] = __digits_tablelookup_build(__digit_chars_upper);
#else
static const char lut_lower[] = "";
static const char lut_upper[] = "";
#endif

static const uint128_t maxu128 = UINT128_MAX;
static const int128_t  maxs128 = INT128_MAX;
static const uint64_t  maxu64  = UINT64_MAX;
static const int64_t   maxs64  = INT64_MAX;
static const uint32_t  maxu32  = UINT32_MAX;
static const int32_t   maxs32  = INT32_MAX;
static const uint16_t  maxu16  = UINT16_MAX;
static const int16_t   maxs16  = INT16_MAX;
static const uint8_t   maxu8   = UINT8_MAX;
static const int8_t    maxs8   = INT8_MAX;

static const int128_t  mins128 = INT128_MIN;
static const int64_t   mins64  = INT64_MIN;
static const int32_t   mins32  = INT32_MIN;
static const int16_t   mins16  = INT16_MIN;
static const int8_t    mins8   = INT8_MIN;
#endif

BOOT_FUNC
static inline void* boot_memcpy(void* restrict dest, const void* restrict src, size_t n)
{
    unsigned char* d = dest;
    const unsigned char* s = src;
    while (n--)
        *d++ = *s++;
    return dest;
}

BOOT_FUNC
static bool is_transmit_empty(enum zerOS_serial_port port)
{
    return zerOS_inb(port + 5) & 0x20;
}

BOOT_FUNC
static void write_serial(enum zerOS_serial_port port, char a)
{
    if (!zerOS_in_qemu() || !zerOS_CONFIG_UNDER_QEMU)
        return;

    while (is_transmit_empty(port) == 0);

    zerOS_outb(port, a);
}

BOOT_FUNC
static void write_debugcon(char a)
{
    if (zerOS_in_qemu() && zerOS_CONFIG_UNDER_QEMU)
        zerOS_outb(0xe9, a);
}

BOOT_FUNC
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
 *    - `%c`: Prints a character.
 *    - `%s`: Prints a null-terminated string.
 *    - `%n`: Prints the number of characters written so far.
 *    - `%%`: Prints a percent sign.
 * @param str The format string.
 * @param varargs The format arguments.
 * @return The number of characters written.
 * @todo Write a proper formatting set of functions.
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
                    static const char basic_lut[] = "0123456789";
                    int64_t value = va_arg(varargs, int64_t);
                    char buffer[64];
                    int i = 0;
                    if (value < 0)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, '-');
                        write_debugcon('-');
                        value = -value;
                        written++;
                    }
                    do
                    {
                        buffer[i++] = basic_lut[value % 10];
                        value /= 10;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'u': {
                    static const char basic_lut[] = "0123456789";
                    uint64_t value = va_arg(varargs, uint64_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut[value % 10];
                        value /= 10;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;                    
                case 'x': {
                    static const char basic_lut_lower[] = "0123456789abcdef";
                    uint64_t value = va_arg(varargs, uint64_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut_lower[value % 16];
                        value /= 16;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'X': {
                    static const char basic_lut_upper[] = "0123456789ABCDEF";
                    uint64_t value = va_arg(varargs, uint64_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut_upper[value % 16];
                        value /= 16;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'p': {
                    static const char basic_lut_lower[] = "0123456789abcdef";
                    uintptr_t value = va_arg(varargs, uintptr_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut_lower[value % 16];
                        value /= 16;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'b': {
                    static const char basic_lut[] = "01";
                    uint64_t value = va_arg(varargs, uint64_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut[value % 2];
                        value /= 2;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'o': {
                    static const char basic_lut[] = "01234567";
                    uint64_t value = va_arg(varargs, uint64_t);
                    char buffer[64];
                    int i = 0;
                    do
                    {
                        buffer[i++] = basic_lut[value % 8];
                        value /= 8;
                    } while (value);
                    while (i--)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, buffer[i]);
                        write_debugcon(buffer[i]);
                        written++;
                    }
                } break;
                case 'c': {
                    char value = va_arg(varargs, int);
                    write_serial(zerOS_SERIAL_DEBUG, value);
                    write_debugcon(value);
                    written++;
                } break;
                case 's': {
                    const char* value = va_arg(varargs, const char*);
                    while (*value)
                    {
                        write_serial(zerOS_SERIAL_DEBUG, *value);
                        write_debugcon(*value);
                        written++;
                        value++;
                    }
                } break;
                case 'n': {
                    BOOT_FUNC PRINTF_LIKE(1, 2)
                    extern int zerOS_early_printk(const char* str, ...);
                    written += zerOS_early_printk("%d", written);
                } break;
                case '%': {
                    write_serial(zerOS_SERIAL_DEBUG, '%');
                    write_debugcon('%');
                    written++;
                } break;

                default: break;
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

BOOT_FUNC
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