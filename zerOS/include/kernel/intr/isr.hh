#ifndef zerOS_KERNEL_INTR_ISR_H_INCLUDED
#define zerOS_KERNEL_INTR_ISR_H_INCLUDED

#include <stdint.h>

typedef void (*zerOS_isr_callback_t)(uint8_t /* intr_nr */);

static_assert(
    sizeof(zerOS_isr_callback_t) * 8 == 64,
    "zerOS_isr_callback_t must be 64 bits wide."
);

#undef  zerOS_ISR_CALLBACKS_MAX
#define zerOS_ISR_CALLBACKS_MAX 32

#endif
