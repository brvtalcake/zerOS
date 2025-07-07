pub fn main()
{
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=tests/ctors.ld");
    //println!("cargo::rustc-link-arg-tests=-Ttests/ctors.ld");
    //println!("cargo::rustc-link-arg-tests=-Ttests/blahblah.ld");
    // rustflags = [
    // "-C", "link-arg=-fuse-ld=lld",
    // ]
}