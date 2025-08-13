#![feature(stmt_expr_attributes)]

#[test]
fn test_expansion()
{
	let option = Some(5);
	let _unwrapped = zerOS_unsafe::block! {
		//! SAFETY: this is safe because of ...
		#[pre(option.is_some())]
		#[post(tmp == 5)]
		let tmp = option.unwrap_unchecked();
		tmp
	};
}
