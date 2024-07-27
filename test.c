int main(void)
{
    int size = ((sizeof(long long) * 8) == 128);
#ifdef __SIZEOF_INT128__
    int typescompatible = (!!__builtin_types_compatible_p(__int128, long long));
    int ret = typescompatible || size;
    return (ret);
#else
    return (size);
#endif
}