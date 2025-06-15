#ifndef zerOS_SPINLOCK_H_INCLUDED_
#define zerOS_SPINLOCK_H_INCLUDED_ 1

#include <stdatomic.h>

#include <zerOS/common.h>

struct zerOS_spinlock
{
	volatile bool locked;
};
static_assert(__atomic_always_lock_free(sizeof(volatile bool), nullptr));

#if 1
#	undef zerOS_SPINLOCK_INITIALIAZER
#	define zerOS_SPINLOCK_INITIALIAZER ((struct zerOS_spinlock){ .locked = false })
#else
static constexpr struct zerOS_spinlock zerOS_SPINLOCK_INITIALIAZER =
  (constexpr struct zerOS_spinlock){ .locked = false };
#endif

static inline bool zerOS_spin_try_lock(struct zerOS_spinlock* spinlock)
{
	return __atomic_exchange_n(&spinlock->locked, true, __ATOMIC_ACQ_REL) == false;
}

static inline void zerOS_spin_lock(struct zerOS_spinlock* spinlock)
{
	// TODO: maybe disable interrupts to avoid deadlocks in case of an interrupt handler calling C
	// code with this kind of basic spin locks

	/*
	 * The following code is mostly a classic "test-and-set" spin lock acquirement.
	 * It is optimized by first testing without any atomic/locking instructions,
	 * to [not saturate the cache](https://geidav.wordpress.com/tag/test-and-test-and-set/)(see also
	 * Wikipedia, on the "Test-and-Test-And-Set" page).
	 * The `_mm_pause();` helps to not have every waiting thread waking up on the newly available
	 * lock at the very same time.
	 */
	do
	{
		while (spinlock->locked)
		{
			zerOS_spin_hint();
			continue;
		}
	} while (!zerOS_spin_try_lock(spinlock));
}

static inline void zerOS_spin_unlock(struct zerOS_spinlock* spinlock)
{
	__atomic_store_n(&spinlock->locked, false, __ATOMIC_RELEASE);
}

static inline void __zerOS_spinlock_guard_cleanup_func(struct zerOS_spinlock** spinlock)
{
	zerOS_spin_unlock(*spinlock);
}

#undef __zerOS_spinlock_guard_impl
#define __zerOS_spinlock_guard_impl(name, spinlock)       \
	[[gnu::cleanup(__zerOS_spinlock_guard_cleanup_func)]] \
	struct zerOS_spinlock* name = (spinlock);             \
	zerOS_spin_lock(name) zerOS_PP_FORCE_SEMICOLON

#endif
