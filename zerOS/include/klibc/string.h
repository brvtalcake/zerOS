#ifndef zerOS_KLIBC_STRING_H_INCLUDED
#define zerOS_KLIBC_STRING_H_INCLUDED

#include <stddef.h>

#if 1

extern
void* memset(void* restrict dest, int c, size_t n);

extern
void* memcpy(void* restrict dest, const void* restrict src, size_t n);

extern
void* memmove(void* dest, const void* src, size_t n);

extern
int memcmp(const void* p1, const void* p2, size_t n);
/* size_t strlen(const char* str);
char *strcpy(char* dest, const char* src);
char *strncpy(char* dest, const char* src, size_t n);
char *strcat(char* dest, const char* src);
char *strncat(char* dest, const char* src, size_t n);
int strcmp(const char* str1, const char* str2);
int strncmp(const char* str1, const char* str2, size_t n);
char *strchr(const char* str, int c);
char *strrchr(const char* str, int c);
char *strstr(const char* haystack, const char* needle);
size_t strspn(const char* str, const char* accept);
size_t strcspn(const char* str, const char* reject);
char *strpbrk(const char* str, const char* accept); */

#else

#define memset(dest, c, n) __builtin_memset(dest, c, n)
#define memcpy(dest, src, n) __builtin_memcpy(dest, src, n)
#define memmove(dest, src, n) __builtin_memmove(dest, src, n)
#define memcmp(ptr1, ptr2, n) __builtin_memcmp(ptr1, ptr2, n)
#define strlen(str) __builtin_strlen(str)
#define strcpy(dest, src) __builtin_strcpy(dest, src)
#define strncpy(dest, src, n) __builtin_strncpy(dest, src, n)
#define strcat(dest, src) __builtin_strcat(dest, src)
#define strncat(dest, src, n) __builtin_strncat(dest, src, n)
#define strcmp(str1, str2) __builtin_strcmp(str1, str2)
#define strncmp(str1, str2, n) __builtin_strncmp(str1, str2, n)
#define strchr(str, c) __builtin_strchr(str, c)
#define strrchr(str, c) __builtin_strrchr(str, c)
#define strstr(haystack, needle) __builtin_strstr(haystack, needle)
#define strspn(str, accept) __builtin_strspn(str, accept)
#define strcspn(str, reject) __builtin_strcspn(str, reject)
#define strpbrk(str, accept) __builtin_strpbrk(str, accept)

#endif

#endif
