#ifndef zerOS_KERNEL_COMPILER_STRUCT_H_INCLUDED
#define zerOS_KERNEL_COMPILER_STRUCT_H_INCLUDED

#undef  sizeof_field
/**
 * @def sizeof_field(type, field)
 * @brief Get the size of a field in a struct.
 * @param type  The type of the struct.
 * @param field The field in the struct.
 */
#define sizeof_field(type, field) sizeof(((type*)0)->field)

#undef  container_of
/**
 * @def container_of(ptr, type, field)
 * @brief Get the container of a field in a struct.
 * @param ptr   The pointer to the field.
 * @param type  The type of the struct.
 * @param field The field in the struct.
 */
#define container_of(ptr, type, field) ((type*)((char*)(ptr) - offsetof(type, field)))

#endif
