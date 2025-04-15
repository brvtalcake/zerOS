#ifndef zerOS_KLIBC_ALLOCA_H_INCLUDED
#define zerOS_KLIBC_ALLOCA_H_INCLUDED

#undef  zalloca
#define zalloca(size)                               \
    ({                                              \
        char* UNIQUE(ptr) = __builtin_alloca(size); \
        for (                                       \
            size_t UNIQUE(i) = 0;                   \
            UNIQUE(i) < size;                       \
            UNIQUE(i)++                             \
        ) UNIQUE(ptr)[UNIQUE(i)] = 0;               \
        (void*)UNIQUE(ptr);                         \
    })

#undef  alloca
#define alloca(size) __builtin_alloca(size)

#endif
