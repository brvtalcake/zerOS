
#[cfg(test)]
mod tests
{
    use proc_macro_utils::array_size;

    #[test]
    fn array_size_test_basic()
    {
        assert_eq!(array_size!([1, 2, 3, 4]), 4);
        assert_eq!(array_size!([   2, 3, 4]), 3);
        assert_eq!(array_size!([      3, 4]), 2);
        assert_eq!(array_size!([         4]), 1);
        assert_eq!(array_size!([          ]), 0);
    }

    #[test]
    fn array_size_test_non_trivial()
    {
        assert_eq!(array_size!([]), 0);
        assert_eq!(array_size!(["1"]), 1);
        assert_eq!(array_size!(["2"]), 1);
        assert_eq!(array_size!(["1, 2, [3]"]), 1);
        assert_eq!(array_size!([1, 2, [3]]), 3);
        assert_eq!(array_size!([[1], [2], [3]]), 3);
    }
}