use proc_macro_utils::ctor;

ctor! {
    
}

ctor! {
    use libc::puts;
    unsafe {
        puts(b"hello world!".as_ptr().cast());
    }
}