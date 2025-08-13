use alloc::{
	alloc::{Allocator, Global},
	boxed::Box,
	sync::Arc
};
use core::{fmt::Debug, mem, ops::CoerceUnsized};

use impls::impls;
use typenum::{Bit, False, True};
use zerOS_static_assertions::static_assert;
use zerOS_sync::BasicRwLock;
use zerocopy::TryFromBytes;

use crate::{
	IOError,
	KernelIO,
	KernelInput,
	KernelOutput,
	KernelPortIO,
	KernelPortInput,
	KernelPortOutput
};

pub struct StreamRead;
pub struct StreamWrite;
pub struct StreamReadWrite;

pub trait StreamDirection
{
	type IsRead: Bit;
	type IsWrite: Bit;
	type Type: ?Sized + Debug + Send + Sync;
	type PortType: ?Sized + Debug + Send + Sync;
}

impl StreamDirection for StreamRead
{
	type IsRead = True;
	type IsWrite = False;
	type PortType = dyn KernelPortInput + Send + Sync;
	type Type = dyn KernelInput + Send + Sync;
}

impl StreamDirection for StreamWrite
{
	type IsRead = False;
	type IsWrite = True;
	type PortType = dyn KernelPortOutput + Send + Sync;
	type Type = dyn KernelOutput + Send + Sync;
}

impl StreamDirection for StreamReadWrite
{
	type IsRead = True;
	type IsWrite = True;
	type PortType = dyn KernelPortIO + Send + Sync;
	type Type = dyn KernelIO + Send + Sync;
}

fn coerce_box<F: ?Sized, T: ?Sized, A: Allocator + Sync + Send>(from: Box<F, A>) -> Box<T, A>
where
	Box<F, A>: CoerceUnsized<Box<T, A>>
{
	from
}

#[derive(Debug, Clone)]
pub struct Wrapper<'a, T: StreamDirection, A: Allocator + Sync + Send = Global>
{
	inner: WrapperInner<'a, T, A>
}

#[derive(Debug, Clone)]
enum WrapperInner<'a, T: StreamDirection, A: Allocator + Sync + Send>
{
	Uninit,
	Basic(Arc<BasicRwLock<Box<<T as StreamDirection>::Type, &'a A>>, &'a A>),
	Port(Arc<BasicRwLock<Box<<T as StreamDirection>::PortType, &'a A>>, &'a A>)
}

impl<'a, T: StreamDirection, A: Allocator + Sync + Send> WrapperInner<'a, T, A>
{
	fn new_basic(boxed: Box<<T as StreamDirection>::Type, &'a A>, allocator: &'a A) -> Self
	{
		Self::Basic(Arc::new_in(BasicRwLock::new(boxed), allocator))
	}

	fn new_port(boxed_port: Box<<T as StreamDirection>::PortType, &'a A>, allocator: &'a A)
	-> Self
	{
		Self::Port(Arc::new_in(BasicRwLock::new(boxed_port), allocator))
	}
}

impl<'a, A: Allocator + Sync + Send> Wrapper<'a, StreamRead, A>
{
	pub fn new_basic_in<B: KernelInput + Send + Sync>(basic: B, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_basic(coerce_box(Box::new_in(basic, allocator)), allocator)
		}
	}

	pub fn new_port_in<P: KernelPortInput + Send + Sync>(port: P, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl<'a, A: Allocator + Sync + Send> Wrapper<'a, StreamWrite, A>
{
	pub fn new_basic_in<B: KernelOutput + Send + Sync>(basic: B, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_basic(coerce_box(Box::new_in(basic, allocator)), allocator)
		}
	}

	pub fn new_port_in<P: KernelPortOutput + Send + Sync>(port: P, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl<'a, A: Allocator + Sync + Send> Wrapper<'a, StreamReadWrite, A>
{
	pub fn new_basic_in<B: KernelIO + Send + Sync>(basic: B, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_basic(coerce_box(Box::new_in(basic, allocator)), allocator)
		}
	}

	pub fn new_port_in<P: KernelPortIO + Send + Sync>(port: P, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl Wrapper<'_, StreamRead, Global>
{
	pub fn new_basic<B: KernelInput + Send + Sync>(basic: B) -> Self
	{
		Self::new_basic_in(basic, &Global)
	}

	pub fn new_port<P: KernelPortInput + Send + Sync>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

impl Wrapper<'_, StreamWrite, Global>
{
	pub fn new_basic<B: KernelOutput + Send + Sync>(basic: B) -> Self
	{
		Self::new_basic_in(basic, &Global)
	}

	pub fn new_port<P: KernelPortOutput + Send + Sync>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

impl Wrapper<'_, StreamReadWrite, Global>
{
	pub fn new_basic<B: KernelIO + Send + Sync>(basic: B) -> Self
	{
		Self::new_basic_in(basic, &Global)
	}

	pub fn new_port<P: KernelPortIO + Send + Sync>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

impl<T: StreamDirection<IsRead = True>, A: Allocator + Sync + Send> Wrapper<'_, T, A>
{
	pub fn read_from_bytes<R: TryFromBytes>(
		&mut self
	) -> Result<([u8; const { mem::size_of::<R>() }], R), IOError>
	{
		todo!()
	}
}

pub type InputStream<'a, A = Global> = Wrapper<'a, StreamRead, A>;
pub type OutputStream<'a, A = Global> = Wrapper<'a, StreamWrite, A>;
pub type IOStream<'a, A = Global> = Wrapper<'a, StreamReadWrite, A>;

static_assert!(impls!(InputStream<'_>: Send & Sync));
static_assert!(impls!(OutputStream<'_>: Send & Sync));
static_assert!(impls!(IOStream<'_>: Send & Sync));
