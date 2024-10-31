#ifndef zerOS_KERNEL_MULTITASKING_ATOMICS_H_INCLUDED
#define zerOS_KERNEL_MULTITASKING_ATOMICS_H_INCLUDED

#include <stdint.h>
#include <stdatomic.h>

#include <machine/path.h>
#include MK_MACHINE_PATH(fast_data_types.h)

typedef atomic_flag zerOS_spinlock_t;
typedef _Atomic(zerOS_fast_uint_t) zerOS_semaphore_t;

#undef  zerOS_SPINLOCK_INIT
#define zerOS_SPINLOCK_INIT ATOMIC_FLAG_INIT

#undef  zerOS_spinlock_acquire
#define zerOS_spinlock_acquire(lock) atomic_flag_test_and_set_explicit(lock, memory_order_acquire)

#undef  zerOS_spinlock_release
#define zerOS_spinlock_release(lock) atomic_flag_clear_explicit(lock, memory_order_release)

#undef  zerOS_SEMAPHORE_INIT
#define zerOS_SEMAPHORE_INIT (0)

#endif
