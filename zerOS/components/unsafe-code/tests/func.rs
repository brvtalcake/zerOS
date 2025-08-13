#![deny(unsafe_code)]
#![feature(never_type)]

/// Unwraps an option without checking
///
/// # Example
/// ```rust
/// # fn main() {
/// let opt = Some(5);
/// assert_eq!(unsafe { unwrap_unchecked(opt) }, 5);
/// # }
/// ```
#[zerOS_unsafe::item(
    safety(
        "The caller must ensure that the option is `Some(_)`"
    ),
    requires(opt.is_some()),
    ensures(ret.is_ok()),
    ensures(old(opt).is_some() -> ret.is_ok())
)]
unsafe fn unwrap_unchecked<T: Copy>(opt: Option<T>) -> Result<T, !>
{
	Ok(unsafe { opt.unwrap_unchecked() })
}
