use core::{
	cmp,
	hint::unreachable_unchecked,
	marker::{Destruct, PointeeSized}
};

use zerOS_macro_utils::min;

/// Like `core::cmp::PartialEq`, but as a `#[const_trait]`
#[const_trait]
pub trait ConstPartialEq<Rhs: PointeeSized = Self>: PointeeSized
{
	/// Tests for `self` and `other` values to be equal, and is used by `==`.
	#[must_use]
	fn const_eq(&self, other: &Rhs) -> bool;

	/// Tests for `!=`. The default implementation is almost always sufficient,
	/// and should not be overridden without very good reason.
	#[inline]
	#[must_use]
	fn const_ne(&self, other: &Rhs) -> bool
	{
		!self.const_eq(other)
	}
}

/// Like `core::cmp::Eq`, but as a `#[const_trait]`
#[const_trait]
pub trait ConstEq: ~const ConstPartialEq<Self> + PointeeSized {}

/// Like `core::cmp::PartialOrd`, but as a `#[const_trait]`
#[const_trait]
pub trait ConstPartialOrd<Rhs: PointeeSized = Self>:
	~const ConstPartialEq<Rhs> + PointeeSized
{
	/// This method returns an ordering between `self` and `other` values if one
	/// exists.
	///
	/// # Examples
	///
	/// ```
	/// use std::cmp::Ordering;
	///
	/// let result = 1.0.partial_cmp(&2.0);
	/// assert_eq!(result, Some(Ordering::Less));
	///
	/// let result = 1.0.partial_cmp(&1.0);
	/// assert_eq!(result, Some(Ordering::Equal));
	///
	/// let result = 2.0.partial_cmp(&1.0);
	/// assert_eq!(result, Some(Ordering::Greater));
	/// ```
	///
	/// When comparison is impossible:
	///
	/// ```
	/// let result = f64::NAN.partial_cmp(&1.0);
	/// assert_eq!(result, None);
	/// ```
	#[must_use]
	fn const_partial_cmp(&self, other: &Rhs) -> Option<cmp::Ordering>;

	/// Tests less than (for `self` and `other`) and is used by the `<`
	/// operator.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.0 < 1.0, false);
	/// assert_eq!(1.0 < 2.0, true);
	/// assert_eq!(2.0 < 1.0, false);
	/// ```
	#[inline]
	#[must_use]
	fn const_lt(&self, other: &Rhs) -> bool
	{
		match self.const_partial_cmp(other)
		{
			Some(value) => value.is_lt(),
			_ => false
		}
	}

	/// Tests less than or equal to (for `self` and `other`) and is used by the
	/// `<=` operator.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.0 <= 1.0, true);
	/// assert_eq!(1.0 <= 2.0, true);
	/// assert_eq!(2.0 <= 1.0, false);
	/// ```
	#[inline]
	#[must_use]
	fn const_le(&self, other: &Rhs) -> bool
	{
		match self.const_partial_cmp(other)
		{
			Some(value) => value.is_le(),
			_ => false
		}
	}

	/// Tests greater than (for `self` and `other`) and is used by the `>`
	/// operator.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.0 > 1.0, false);
	/// assert_eq!(1.0 > 2.0, false);
	/// assert_eq!(2.0 > 1.0, true);
	/// ```
	#[inline]
	#[must_use]
	fn const_gt(&self, other: &Rhs) -> bool
	{
		match self.const_partial_cmp(other)
		{
			Some(value) => value.is_gt(),
			_ => false
		}
	}

	/// Tests greater than or equal to (for `self` and `other`) and is used by
	/// the `>=` operator.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.0 >= 1.0, true);
	/// assert_eq!(1.0 >= 2.0, false);
	/// assert_eq!(2.0 >= 1.0, true);
	/// ```
	#[inline]
	#[must_use]
	fn const_ge(&self, other: &Rhs) -> bool
	{
		match self.const_partial_cmp(other)
		{
			Some(value) => value.is_ge(),
			_ => false
		}
	}
}

/// Like `core::cmp::Ord`, but as a `#[const_trait]`
#[const_trait]
pub trait ConstOrd: ~const ConstEq + ~const ConstPartialOrd<Self> + PointeeSized
{
	/// This method returns an [`Ordering`] between `self` and `other`.
	///
	/// By convention, `self.cmp(&other)` returns the ordering matching the
	/// expression `self <operator> other` if true.
	///
	/// # Examples
	///
	/// ```
	/// use std::cmp::Ordering;
	///
	/// assert_eq!(5.cmp(&10), Ordering::Less);
	/// assert_eq!(10.cmp(&5), Ordering::Greater);
	/// assert_eq!(5.cmp(&5), Ordering::Equal);
	/// ```
	#[must_use]
	fn const_cmp(&self, other: &Self) -> cmp::Ordering;

	/// Compares and returns the maximum of two values.
	///
	/// Returns the second argument if the comparison determines them to be
	/// equal.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.max(2), 2);
	/// assert_eq!(2.max(2), 2);
	/// ```
	/// ```
	/// use std::cmp::Ordering;
	///
	/// #[derive(Eq)]
	/// struct Equal(&'static str);
	///
	/// impl PartialEq for Equal
	/// {
	/// 	fn eq(&self, other: &Self) -> bool
	/// 	{
	/// 		true
	/// 	}
	/// }
	/// impl PartialOrd for Equal
	/// {
	/// 	fn partial_cmp(&self, other: &Self) -> Option<Ordering>
	/// 	{
	/// 		Some(Ordering::Equal)
	/// 	}
	/// }
	/// impl Ord for Equal
	/// {
	/// 	fn cmp(&self, other: &Self) -> Ordering
	/// 	{
	/// 		Ordering::Equal
	/// 	}
	/// }
	///
	/// assert_eq!(Equal("self").max(Equal("other")).0, "other");
	/// ```
	#[inline]
	#[must_use]
	fn const_max(self, other: Self) -> Self
	where
		Self: Sized + ~const Destruct
	{
		if self.const_gt(&other) { self } else { other }
	}

	/// Compares and returns the minimum of two values.
	///
	/// Returns the first argument if the comparison determines them to be
	/// equal.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!(1.min(2), 1);
	/// assert_eq!(2.min(2), 2);
	/// ```
	/// ```
	/// use std::cmp::Ordering;
	///
	/// #[derive(Eq)]
	/// struct Equal(&'static str);
	///
	/// impl PartialEq for Equal
	/// {
	/// 	fn eq(&self, other: &Self) -> bool
	/// 	{
	/// 		true
	/// 	}
	/// }
	/// impl PartialOrd for Equal
	/// {
	/// 	fn partial_cmp(&self, other: &Self) -> Option<Ordering>
	/// 	{
	/// 		Some(Ordering::Equal)
	/// 	}
	/// }
	/// impl Ord for Equal
	/// {
	/// 	fn cmp(&self, other: &Self) -> Ordering
	/// 	{
	/// 		Ordering::Equal
	/// 	}
	/// }
	///
	/// assert_eq!(Equal("self").min(Equal("other")).0, "self");
	/// ```
	#[inline]
	#[must_use]
	fn const_min(self, other: Self) -> Self
	where
		Self: Sized + ~const Destruct
	{
		if self.const_lt(&other) { self } else { other }
	}

	/// Restrict a value to a certain interval.
	///
	/// Returns `max` if `self` is greater than `max`, and `min` if `self` is
	/// less than `min`. Otherwise this returns `self`.
	///
	/// # Panics
	///
	/// Panics if `min > max`.
	///
	/// # Examples
	///
	/// ```
	/// assert_eq!((-3).clamp(-2, 1), -2);
	/// assert_eq!(0.clamp(-2, 1), 0);
	/// assert_eq!(2.clamp(-2, 1), 1);
	/// ```
	#[must_use]
	#[inline]
	fn const_clamp(self, min: Self, max: Self) -> Self
	where
		Self: Sized + ~const Destruct
	{
		assert!(min.const_le(&max));
		if self.const_lt(&min)
		{
			min
		}
		else if self.const_gt(&max)
		{
			max
		}
		else
		{
			self
		}
	}
}

macro_rules! impl_partial_eq {
    ($($t:ty)*) => {
        $(
            impl const ConstPartialEq for $t
            {
                fn const_eq(&self, other: &Self) -> bool
                {
                    *self == *other
                }
            }
        )*
    };
}

macro_rules! impl_eq {
    ($($t:ty)*) => {
        $(
            impl const ConstEq for $t {}
        )*
    };
}

macro_rules! impl_partial_ord {
    ($($t:ty)*) => {
        $(
            impl const ConstPartialOrd for $t
            {
                fn const_partial_cmp(&self, other: &Self) -> Option<cmp::Ordering>
                {
                    match (*self <= *other, *self >= *other)
                    {
                        (false, false) => None,
                        (false, true) => Some(cmp::Ordering::Greater),
                        (true, false) => Some(cmp::Ordering::Less),
                        (true, true) => Some(cmp::Ordering::Equal)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_ord {
    ($($t:ty)*) => {
        $(
            impl const ConstOrd for $t
            {
                fn const_cmp(&self, other: &Self) -> cmp::Ordering {
                    core::intrinsics::three_way_compare(*self, *other)
                }
            }
            impl_partial_ord! { $t }
        )*
    };
}

impl const ConstPartialEq for ()
{
	#[inline]
	fn const_eq(&self, _other: &()) -> bool
	{
		true
	}

	#[inline]
	fn const_ne(&self, _other: &()) -> bool
	{
		false
	}
}

impl const ConstPartialOrd for ()
{
	#[inline]
	fn const_partial_cmp(&self, _: &()) -> Option<cmp::Ordering>
	{
		Some(cmp::Ordering::Equal)
	}
}

impl const ConstOrd for ()
{
	#[inline]
	fn const_cmp(&self, _other: &()) -> cmp::Ordering
	{
		cmp::Ordering::Equal
	}
}

impl const ConstPartialOrd for bool
{
	#[inline]
	fn const_partial_cmp(&self, other: &bool) -> Option<cmp::Ordering>
	{
		Some(self.const_cmp(other))
	}
}

impl const ConstOrd for bool
{
	#[inline]
	fn const_cmp(&self, other: &bool) -> cmp::Ordering
	{
		// Casting to i8's and converting the difference to an Ordering generates
		// more optimal assembly.
		// See <https://github.com/rust-lang/rust/issues/66780> for more info.
		match (*self as i8) - (*other as i8)
		{
			-1 => cmp::Ordering::Less,
			0 => cmp::Ordering::Equal,
			1 => cmp::Ordering::Greater,
			// SAFETY: bool as i8 returns 0 or 1, so the difference can't be anything else
			_ =>
			unsafe { unreachable_unchecked() }
		}
	}

	#[inline]
	fn const_min(self, other: bool) -> bool
	{
		self & other
	}

	#[inline]
	fn const_max(self, other: bool) -> bool
	{
		self | other
	}

	#[inline]
	fn const_clamp(self, min: bool, max: bool) -> bool
	{
		assert!(min.const_le(&max));
		self.const_max(min).const_min(max)
	}
}

impl const ConstPartialEq for !
{
	#[inline]
	fn const_eq(&self, _: &!) -> bool
	{
		*self
	}
}

impl const ConstEq for ! {}

impl const ConstPartialOrd for !
{
	#[inline]
	fn const_partial_cmp(&self, _: &!) -> Option<cmp::Ordering>
	{
		*self
	}
}

impl const ConstOrd for !
{
	#[inline]
	fn const_cmp(&self, _: &!) -> cmp::Ordering
	{
		*self
	}
}

impl const ConstPartialEq for str
{
	#[inline]
	fn const_eq(&self, other: &str) -> bool
	{
		let len = self.as_bytes().len();
		if other.as_bytes().len() != len
		{
			return false;
		}
		unsafe {
			core::intrinsics::compare_bytes(
				self.as_bytes().as_ptr(),
				other.as_bytes().as_ptr(),
				len
			) == 0
		}
	}
}

impl const ConstEq for str {}

impl const ConstPartialOrd for str
{
	#[inline]
	fn const_partial_cmp(&self, other: &str) -> Option<cmp::Ordering>
	{
		let (self_bytes, other_bytes) = (self.as_bytes(), other.as_bytes());
		let (self_len, other_len) = (self_bytes.len(), other_bytes.len());
		unsafe {
			Some({
				let res = core::intrinsics::compare_bytes(
					self_bytes.as_ptr(),
					other_bytes.as_ptr(),
					min!(self_len, other_len)
				);
				if res < 0
				{
					cmp::Ordering::Less
				}
				else if res > 0
				{
					cmp::Ordering::Greater
				}
				else
				{
					// TODO: is it even true ?
					self_len.const_partial_cmp(&other_len).unwrap_unchecked()
				}
			})
		}
	}
}

impl const ConstOrd for str
{
	#[inline]
	fn const_cmp(&self, other: &str) -> cmp::Ordering
	{
		unsafe { self.const_partial_cmp(other).unwrap_unchecked() }
	}
}

impl_partial_eq! { bool char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 f16 f32 f64 f128 }
impl_eq! { () bool char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }
impl_partial_ord! { f16 f32 f64 f128 }
impl_ord! { char usize u8 u16 u32 u64 u128 isize i8 i16 i32 i64 i128 }

impl<A: PointeeSized, B: PointeeSized> const ConstPartialEq<&B> for &A
where
	A: ~const ConstPartialEq<B>
{
	#[inline]
	fn const_eq(&self, other: &&B) -> bool
	{
		ConstPartialEq::const_eq(*self, *other)
	}
}

impl<A: PointeeSized, B: PointeeSized> const ConstPartialOrd<&B> for &A
where
	A: ~const ConstPartialOrd<B>
{
	#[inline]
	fn const_partial_cmp(&self, other: &&B) -> Option<cmp::Ordering>
	{
		ConstPartialOrd::const_partial_cmp(*self, *other)
	}
}

impl<A: PointeeSized> const ConstOrd for &A
where
	A: ~const ConstOrd
{
	#[inline]
	fn const_cmp(&self, other: &Self) -> cmp::Ordering
	{
		ConstOrd::const_cmp(*self, *other)
	}
}
impl<A: PointeeSized> const ConstEq for &A where A: ~const ConstEq {}

// &mut pointers

impl<A: PointeeSized, B: PointeeSized> const ConstPartialEq<&mut B> for &mut A
where
	A: ~const ConstPartialEq<B>
{
	#[inline]
	fn const_eq(&self, other: &&mut B) -> bool
	{
		ConstPartialEq::const_eq(*self, *other)
	}
}

impl<A: PointeeSized, B: PointeeSized> const ConstPartialOrd<&mut B> for &mut A
where
	A: ~const ConstPartialOrd<B>
{
	#[inline]
	fn const_partial_cmp(&self, other: &&mut B) -> Option<cmp::Ordering>
	{
		ConstPartialOrd::const_partial_cmp(*self, *other)
	}
}

impl<A: PointeeSized> const ConstOrd for &mut A
where
	A: ~const ConstOrd
{
	#[inline]
	fn const_cmp(&self, other: &Self) -> cmp::Ordering
	{
		ConstOrd::const_cmp(*self, *other)
	}
}
impl<A: PointeeSized> const ConstEq for &mut A where A: ~const ConstEq {}

impl<A: PointeeSized, B: PointeeSized> const ConstPartialEq<&mut B> for &A
where
	A: ~const ConstPartialEq<B>
{
	#[inline]
	fn const_eq(&self, other: &&mut B) -> bool
	{
		ConstPartialEq::const_eq(*self, *other)
	}
}

impl<A: PointeeSized, B: PointeeSized> const ConstPartialEq<&B> for &mut A
where
	A: ~const ConstPartialEq<B>
{
	#[inline]
	fn const_eq(&self, other: &&B) -> bool
	{
		ConstPartialEq::const_eq(*self, *other)
	}
}
