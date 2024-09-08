#ifndef zerOS_MISC_TYPE_H_INCLUDED
#define zerOS_MISC_TYPE_H_INCLUDED

#undef  TYPE_PACKED
/**
 * @def TYPE_PACKED
 * @brief Specify that a type (struct or union) has a packed layout.
 * 
 */
#define TYPE_PACKED [[__gnu__::__packed__]]

#undef  TYPE_ALIGNED_TO
/**
 * @def TYPE_ALIGNED_TO
 * @brief Specify that a type is aligned to a specified alignment.
 * 
 */
#define TYPE_ALIGNED_TO(alignment) [[__gnu__::__aligned__(alignment)]]

#endif
