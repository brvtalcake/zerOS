#ifndef zerOS_GUARD_H_INCLUDED_
#define zerOS_GUARD_H_INCLUDED_ 1

#undef zerOS_guard
#define zerOS_guard(kind) __zerOS_##kind##_guard_impl

#endif
