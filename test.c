#include <stddef.h>
#include <stdint.h>
#include <stdio.h>


// uintX_t __absX(intX_t v) (pseudocode):
// {
//     uintX_t r;
//     const intX_t mask = v >> (sizeof(intX_t) * 8 - 1);
//     r = (v ^ mask) - mask;
//     return r;
// }
// (use SyS-V ABI)
asm (
    ".section .text\n"
    ".global __abs8\n"
    ".type __abs8, @function\n"
    ".balign 16\n"
    "__abs8:\n"
    "    mov %edi, %edx\n"
    "    sar $0x7, %dl\n"
    "    xor %dl, %dil\n"
);
extern uint8_t __abs8(int8_t v);

extern uint8_t __abs8_gcc(int8_t v)
{
    uint8_t r;
    const int8_t mask = v >> 7;
    r = (v ^ mask) - mask;
    return r;
}

int main(int argc, char const *argv[])
{
    int8_t v = -128;
    printf("abs(%d) = %u\n", v, __abs8(v));
    printf("abs_gcc(%d) = %u\n", v, __abs8_gcc(v));
    return 0;
}
