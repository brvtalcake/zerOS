void f1();
void f2();

[[gnu::noinline]]
void f2()
{
    __builtin_printf("    real : %p\n", &f1);
    __builtin_printf("measured : %p\n", __builtin_extract_return_addr(__builtin_return_address(0)));
}

[[gnu::noinline]]
void f1()
{
    f2();
}

int main(void)
{
#if 0
    int size = ((sizeof(long long) * 8) == 128);
#ifdef __SIZEOF_INT128__
    int typescompatible = (!!__builtin_types_compatible_p(__int128, long long));
    int ret = typescompatible || size;
    return (ret);
#else
    return (size);
#endif
#else
    f1();
#endif
}