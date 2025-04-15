#ifndef zerOS_MACHINE_DFDM_H_INCLUDED
#define zerOS_MACHINE_DFDM_H_INCLUDED

#include <config.h>
#include <chaos/preprocessor/cat.h>

#undef  MACHINE_DISPLAY_FAMILY
#define MACHINE_DISPLAY_FAMILY CHAOS_PP_CAT(zerOS_CONFIG_CPU, _DISPLAY_FAMILY)

#undef  MACHINE_DISPLAY_MODEL
#define MACHINE_DISPLAY_MODEL CHAOS_PP_CAT(zerOS_CONFIG_CPU, _DISPLAY_MODEL)

#undef  alderlake_DISPLAY_FAMILY
#undef  alderlake_DISPLAY_MODEL
#define alderlake_DISPLAY_FAMILY 0x06
#define alderlake_DISPLAY_MODEL 0x97, 0x9a

#endif
