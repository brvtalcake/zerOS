#ifndef zerOS_MACHINE_PATH_H_INCLUDED
#define zerOS_MACHINE_PATH_H_INCLUDED

#include <config.h>

#undef  MACHINE_PATH
#define MACHINE_PATH machine/zerOS_CONFIG_CPU

#undef  MK_MACHINE_PATH
#define MK_MACHINE_PATH(file) <MACHINE_PATH/file>

#endif
