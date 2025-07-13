use alloc::{
	alloc::{Allocator, Global},
	boxed::Box,
	sync::Arc
};
use core::ops::CoerceUnsized;

use typenum::{Bit, False, True};
use zerOS_sync::BasicRwLock;

use crate::{KernelIO, KernelInput, KernelOutput, KernelPortIO, KernelPortInput, KernelPortOutput};

pub struct StreamRead;
pub struct StreamWrite;
pub struct StreamReadWrite;

pub mod traits
{
	use crate::{
		KernelIO,
		KernelInput,
		KernelOutput,
		KernelPortIO,
		KernelPortInput,
		KernelPortOutput
	};

	macro autoimpl(
        $(
            $trait:ident ( $($tokens:tt)+ )
        ),* $(,)?
    ) {
        $(
            pub trait $trait: $($tokens)+
            {
            }
            impl<T> $trait for T
                where T: $($tokens)+
            {
            }
        )*
    }
	autoimpl! {
		KPortIn ( KernelInput + KernelPortInput ),
		KPortOut ( KernelOutput + KernelPortOutput ),
		KPortIO ( KernelIO + KernelPortIO ),
	}
}
use traits::*;

pub trait StreamDirection
{
	type IsRead: Bit;
	type IsWrite: Bit;
	type Type: ?Sized;
	type PortType: ?Sized;
}

impl StreamDirection for StreamRead
{
	type IsRead = True;
	type IsWrite = False;
	type PortType = dyn KPortIn + Send;
	type Type = dyn KernelInput + Send;
}

impl StreamDirection for StreamWrite
{
	type IsRead = False;
	type IsWrite = True;
	type PortType = dyn KPortOut + Send;
	type Type = dyn KernelOutput + Send;
}

impl StreamDirection for StreamReadWrite
{
	type IsRead = True;
	type IsWrite = True;
	type PortType = dyn KPortIO + Send;
	type Type = dyn KernelIO + Send;
}

fn coerce_box<F: ?Sized, T: ?Sized, A: Allocator>(from: Box<F, A>) -> Box<T, A>
where
	Box<F, A>: CoerceUnsized<Box<T, A>>
{
	from
}

pub struct Wrapper<'a, T: StreamDirection, A: Allocator = Global>
{
	inner: WrapperInner<'a, T, A>
}

enum WrapperInner<'a, T: StreamDirection, A: Allocator>
{
	Uninit,
	Port(Arc<BasicRwLock<Box<<T as StreamDirection>::PortType, &'a A>>, &'a A>)
}

impl<'a, T: StreamDirection, A: Allocator> WrapperInner<'a, T, A>
{
	fn new_port(boxed_port: Box<<T as StreamDirection>::PortType, &'a A>, allocator: &'a A)
	-> Self
	{
		Self::Port(Arc::new_in(BasicRwLock::new(boxed_port), allocator))
	}
}

impl<'a, A: Allocator> Wrapper<'a, StreamRead, A>
{
	pub fn new_port_in<P: KernelInput + KernelPortInput + Send>(port: P, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl<'a, A: Allocator> Wrapper<'a, StreamWrite, A>
{
	pub fn new_port_in<P: KernelOutput + KernelPortOutput + Send>(port: P, allocator: &'a A)
	-> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl<'a, A: Allocator> Wrapper<'a, StreamReadWrite, A>
{
	pub fn new_port_in<P: KernelIO + KernelPortIO + Send>(port: P, allocator: &'a A) -> Self
	{
		Self {
			inner: WrapperInner::new_port(coerce_box(Box::new_in(port, allocator)), allocator)
		}
	}
}

impl<T: StreamDirection<IsRead = True>, A: Allocator> Wrapper<'_, T, A>
{
	fn read(&self) {}
}

impl Wrapper<'_, StreamRead, Global>
{
	pub fn new_port<P: KernelInput + KernelPortInput + Send>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

impl Wrapper<'_, StreamWrite, Global>
{
	pub fn new_port<P: KernelOutput + KernelPortOutput + Send>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

impl Wrapper<'_, StreamReadWrite, Global>
{
	pub fn new_port<P: KernelIO + KernelPortIO + Send>(port: P) -> Self
	{
		Self::new_port_in(port, &Global)
	}
}

pub type InputStream<'a, A = Global> = Wrapper<'a, StreamRead, A>;
pub type OutputStream<'a, A = Global> = Wrapper<'a, StreamWrite, A>;
pub type IOStream<'a, A = Global> = Wrapper<'a, StreamReadWrite, A>;
