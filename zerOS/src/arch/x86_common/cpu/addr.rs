use core::{
	mem::MaybeUninit,
	ptr::{self, NonNull}
};

use num::cast::AsPrimitive;

use crate::width_of;

fn zero_extend_impl(bits: usize) -> impl FnOnce(usize) -> usize
{
	move |addr| {
		let mask = (1 << bits) - 1;
		addr & mask
	}
}

fn sign_extend_impl(bits: usize) -> impl FnOnce(usize) -> usize
{
	move |addr| {
		let u64addr: u64 = addr.as_();
		let shift = width_of!(usize) - bits;
		(((u64addr << shift) as i64 >> shift) as u64).as_()
	}
}

pub mod phys
{
	use overloadable::overloadable;

	use super::{super::zerOS_boot_cpu_physical_address_bits, sign_extend_impl, zero_extend_impl};

	overloadable! {
		pub canonical as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			zero_extend(ptr)
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			zero_extend(ptr)
		}
	}

	overloadable! {
		pub zero_extend as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			ptr.map_addr(zero_extend_impl(unsafe {
				zerOS_boot_cpu_physical_address_bits
			}))
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			ptr.map_addr(zero_extend_impl(unsafe {
				zerOS_boot_cpu_physical_address_bits
			}))
		}
	}

	overloadable! {
		pub sign_extend as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			ptr.map_addr(sign_extend_impl(unsafe {
				zerOS_boot_cpu_physical_address_bits
			}))
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			ptr.map_addr(sign_extend_impl(unsafe {
				zerOS_boot_cpu_physical_address_bits
			}))
		}
	}
}

pub mod virt
{
	use overloadable::overloadable;

	use super::{super::zerOS_boot_cpu_linear_address_bits, sign_extend_impl, zero_extend_impl};

	overloadable! {
		pub canonical as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			sign_extend(ptr)
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			sign_extend(ptr)
		}
	}

	overloadable! {
		pub zero_extend as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			ptr.map_addr(zero_extend_impl(unsafe {
				zerOS_boot_cpu_linear_address_bits
			}))
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			ptr.map_addr(zero_extend_impl(unsafe {
				zerOS_boot_cpu_linear_address_bits
			}))
		}
	}

	overloadable! {
		pub sign_extend as
		fn<T: ?Sized>(ptr: *mut T) -> *mut T
		{
			ptr.map_addr(sign_extend_impl(unsafe {
				zerOS_boot_cpu_linear_address_bits
			}))
		},
		fn<T: ?Sized>(ptr: *const T) -> *const T
		{
			ptr.map_addr(sign_extend_impl(unsafe {
				zerOS_boot_cpu_linear_address_bits
			}))
		}
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VirtAddrMut<T: ?Sized>
{
	addr: *mut T
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct VirtAddrConst<T: ?Sized>
{
	addr: *const T
}

impl<T: ?Sized> VirtAddrMut<T>
{
	#[inline]
	pub const unsafe fn new_unchecked(pointer: *mut T) -> Self
	{
		Self { addr: pointer }
	}

	#[inline]
	pub fn canonical(pointer: *mut T) -> Self
	{
		unsafe { Self::new_unchecked(virt::canonical(pointer)) }
	}

	#[inline]
	#[must_use]
	pub fn raw(self) -> *mut T
	{
		self.addr
	}
}

impl<T: ?Sized> From<*mut T> for VirtAddrMut<T>
{
	#[inline]
	fn from(value: *mut T) -> Self
	{
		Self::canonical(value)
	}
}

impl<T: ?Sized> From<NonNull<T>> for VirtAddrMut<T>
{
	#[inline]
	fn from(value: NonNull<T>) -> Self
	{
		Self::canonical(value.as_ptr())
	}
}

impl<T: ?Sized> VirtAddrConst<T>
{
	#[inline]
	pub const unsafe fn new_unchecked(pointer: *const T) -> Self
	{
		Self { addr: pointer }
	}

	#[inline]
	pub fn canonical(pointer: *const T) -> Self
	{
		unsafe { Self::new_unchecked(virt::canonical(pointer)) }
	}

	#[inline]
	#[must_use]
	pub fn raw(self) -> *const T
	{
		self.addr
	}
}

impl<T: ?Sized> From<*const T> for VirtAddrConst<T>
{
	#[inline]
	fn from(value: *const T) -> Self
	{
		Self::canonical(value)
	}
}

impl<T: ?Sized> From<NonNull<T>> for VirtAddrConst<T>
{
	#[inline]
	fn from(value: NonNull<T>) -> Self
	{
		Self::canonical(value.as_ptr().cast_const())
	}
}

impl<T: ?Sized> VirtAddrMut<T>
{
	#[inline]
	pub const fn is_null(self) -> bool
	{
		self.addr.is_null()
	}

	#[inline(always)]
	pub fn cast<U>(self) -> VirtAddrMut<U>
	{
		self.addr.cast::<U>().into()
	}

	#[must_use = "this returns the result of the operation, without modifying the original"]
	#[inline]
	pub fn try_cast_aligned<U>(self) -> Option<VirtAddrMut<U>>
	{
		self.addr.try_cast_aligned().map(VirtAddrMut::from)
	}

	#[must_use = "returns a new pointer rather than modifying its argument"]
	#[inline]
	pub fn with_metadata_of<U>(self, meta: *const U) -> VirtAddrMut<U>
	where
		U: ?Sized
	{
		VirtAddrMut::from(self.addr.with_metadata_of(meta))
	}

	#[inline(always)]
	pub const fn cast_const(self) -> VirtAddrConst<T>
	{
		// SAFETY: the value didn't change
		unsafe { VirtAddrConst::new_unchecked(self.addr.cast_const()) }
	}

	#[must_use]
	#[inline(always)]
	pub fn addr(self) -> usize
	{
		self.addr.addr()
	}

	#[inline(always)]
	pub fn expose_provenance(self) -> usize
	{
		self.addr.expose_provenance()
	}

	#[must_use]
	#[inline]
	pub fn with_addr(self, other: usize) -> Self
	{
		self.addr.with_addr(other).into()
	}

	#[must_use]
	#[inline]
	pub fn map_addr(self, f: impl FnOnce(usize) -> usize) -> Self
	{
		self.addr.map_addr(f).into()
	}

	#[inline]
	pub fn to_raw_parts(self) -> (VirtAddrMut<()>, <T as ptr::Pointee>::Metadata)
	{
		let (ptr, metadata) = self.addr.to_raw_parts();
		(ptr.into(), metadata)
	}

	#[inline]
	pub const unsafe fn as_ref<'a>(self) -> Option<&'a T>
	{
		unsafe { self.addr.as_ref() }
	}

	#[inline]
	#[must_use]
	pub const unsafe fn as_ref_unchecked<'a>(self) -> &'a T
	{
		unsafe { self.addr.as_ref_unchecked() }
	}

	#[inline]
	pub const unsafe fn as_uninit_ref<'a>(self) -> Option<&'a MaybeUninit<T>>
	where
		T: Sized
	{
		unsafe { self.addr.as_uninit_ref() }
	}

	#[must_use = "returns a new pointer rather than modifying its argument"]
	#[inline(always)]
	#[track_caller]
	pub unsafe fn offset(self, count: isize) -> Self
	where
		T: Sized
	{
		unsafe { self.addr.offset(count) }.into()
	}

	#[must_use]
	#[inline(always)]
	#[track_caller]
	pub unsafe fn byte_offset(self, count: isize) -> Self
	{
		unsafe { self.addr.byte_offset(count) }.into()
	}

	#[must_use = "returns a new pointer rather than modifying its argument"]
	#[inline(always)]
	pub fn wrapping_offset(self, count: isize) -> Self
	where
		T: Sized
	{
		self.addr.wrapping_offset(count).into()
	}

	#[must_use]
	#[inline(always)]
	pub fn wrapping_byte_offset(self, count: isize) -> Self
	{
		self.addr.wrapping_byte_offset(count).into()
	}

	#[must_use = "returns a new pointer rather than modifying its argument"]
	#[inline(always)]
	pub fn mask(self, mask: usize) -> Self
	{
		self.addr.mask(mask).into()
	}

	#[inline]
	pub const unsafe fn as_mut<'a>(self) -> Option<&'a mut T>
	{
		unsafe { self.addr.as_mut() }
	}

	#[inline]
	#[must_use]
	pub const unsafe fn as_mut_unchecked<'a>(self) -> &'a mut T
	{
		unsafe { self.addr.as_mut_unchecked() }
	}

	#[inline]
	pub const unsafe fn as_uninit_mut<'a>(self) -> Option<&'a mut MaybeUninit<T>>
	where
		T: Sized
	{
		unsafe { self.addr.as_uninit_mut() }
	}

	#[inline]
	pub fn guaranteed_eq<O: Into<Self>>(self, other: O) -> Option<bool>
	where
		T: Sized
	{
		let other_self: Self = other.into();
		self.addr.guaranteed_eq(other_self.raw())
	}

	#[inline]
	pub fn guaranteed_ne<O: Into<Self>>(self, other: O) -> Option<bool>
	where
		T: Sized
	{
		let other_self: Self = other.into();
		self.addr.guaranteed_ne(other_self.raw())
	}

	#[inline(always)]
	#[cfg_attr(miri, track_caller)] // even without panics, this helps for Miri backtraces
	pub const unsafe fn offset_from(self, origin: *const T) -> isize
	where
		T: Sized
	{
		unsafe { self.addr.offset_from(origin) }
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct PhysAddr<T: ?Sized>
{
	addr: *mut T
}
