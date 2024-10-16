// All functions here are following SysV ABI calling convention

.include "asm/gas/common.inc"

#undef  ASM_SYNTAX
#define ASM_SYNTAX ASM_SYNTAX_ATT

#include <asm/syntax.h>
#include <asm/common.h>

#include <misc/sections.h>

IN_SECTION(.bootcode)
EXTERN_FUNC(__early_printk_u64_to_string)
/*
 * void __early_printk_u64_to_string(uint64_t x, char *buf)
 * Algortihm:
 *   h = x/100000000; l = x%100000000;
 *
 *   hh = (h * 109951163) >> 40; h -= hh * 10000; h |= hh << 32;
 *   ll = (l * 109951163) >> 40; l -= ll * 10000; l |= ll << 32;
 *
 *   hh = ((h * 5243) >> 19) & 0x000000FF000000FF; h -= hh * 100; h |= hh << 16;
 *   ll = ((l * 5243) >> 19) & 0x000000FF000000FF; l -= ll * 100; l |= ll << 16;
 *
 *   hh = ((h * 103) >> 9) & 0x001E001E001E001E; h += hh * 3;
 *   ll = ((l * 103) >> 9) & 0x001E001E001E001E; l += ll * 3;
 *
 *   hh = (h & 0x00FF000000FF0000l); h ^= hh ^ (hh >> 8);  // 0x0000aabb0000ccdd
 *   ll = (l & 0x00FF000000FF0000l); l ^= ll ^ (ll >> 8);  // 0x0000aabb0000ccdd
 *
 *   hh = (h & 0x0000FFFF00000000l); h ^= hh ^ (hh >> 16); // 0x00000000aabbccdd
 *   ll = (l & 0x0000FFFF00000000l); l ^= ll ^ (ll >> 16); // 0x00000000aabbccdd
 *
 *   return l|(h<<32);
 */