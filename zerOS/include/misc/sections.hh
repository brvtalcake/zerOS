#ifndef zerOS_MISC_SECTIONS_H_INCLUDED
#define zerOS_MISC_SECTIONS_H_INCLUDED

#ifndef __ASSEMBLER__

#undef  IN_SECTION
/**
 * @def IN_SECTION(section_name)
 * @brief Marks a variable as being in a specific section.
 * @param section_name The name of the section.
 */
#define IN_SECTION(section_name) [[__gnu__::__section__(section_name)]]

#undef  BOOT_FUNC
/**
 * @def BOOT_FUNC
 * @brief Marks a function as being a boot function.
 * @todo Add some attributes to disable ISA extensions
 */
#define BOOT_FUNC IN_SECTION(".bootcode") [[__gnu__::__optimize__("no-lto")]]

#else

#include <asm/syntax.h>

#undef  IN_SECTION
#define IN_SECTION(section_name) .section section_name

#undef  BOOT_FUNC
#define BOOT_FUNC /* TODO */

#endif

#endif
