#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(trait_alias)]

use core::fmt;

use zerOS_macro_utils::max;

pub trait SerialInput
{
	fn serial_read_byte(&self) -> u8;

	fn serial_read_bytes(&self, bytes: &mut [u8], max: Option<usize>)
	{
		if let Some(max_written) = max
		{
			let absmax = bytes.len();
			for i in 0..(max!(max_written, absmax))
			{
				bytes[i] = self.serial_read_byte();
			}
		}
		else
		{
			for b in bytes
			{
				*b = self.serial_read_byte();
			}
		}
	}

	fn serial_read_bytes_while(
		&self,
		bytes: &mut [u8],
		mut predicate: impl FnMut(&u8) -> bool
	) -> usize
	{
		let (mut i, absmax): (usize, usize) = (0, bytes.len());
		while i < absmax
		{
			let buf = self.serial_read_byte();
			if !predicate(&buf)
			{
				return i;
			}
			bytes[i] = buf;
			i += 1;
		}
		i
	}
	fn serial_read_bytes_until(
		&self,
		bytes: &mut [u8],
		mut predicate: impl FnMut(&u8) -> bool
	) -> usize
	{
		self.serial_read_bytes_while(bytes, |val| !predicate(val))
	}
}

pub trait SerialOutput
{
	fn supports_ansi_escape_codes(&self) -> bool
	{
		false
	}

	fn serial_write_byte(&self, byte: u8);

	fn serial_write_bytes(&self, bytes: &[u8])
	{
		for b in bytes
		{
			self.serial_write_byte(*b);
		}
	}
}

pub trait SerialIO = SerialInput + SerialOutput;

pub struct SerialKernelOutput<T: SerialOutput>
{
	inner: T
}

impl<T: SerialOutput> SerialKernelOutput<T>
{
	pub const fn new(inner: T) -> Self
	{
		Self { inner }
	}
}

impl<T: SerialOutput> fmt::Write for SerialKernelOutput<T>
{
	fn write_str(&mut self, s: &str) -> fmt::Result
	{
		self.inner.serial_write_bytes(s.as_bytes());
		Ok(())
	}
}

impl<T: SerialOutput> KernelOutput for SerialKernelOutput<T>
{
	fn flush(&mut self) -> fmt::Result
	{
		Ok(())
	}

	fn supports_ansi_escape_codes(&self) -> bool
	{
		self.inner.supports_ansi_escape_codes()
	}

	fn write_byte(&self, byte: u8)
	{
		self.inner.serial_write_byte(byte);
	}

	fn write_bytes(&self, bytes: &[u8])
	{
		self.inner.serial_write_bytes(bytes);
	}
}

impl<T: SerialOutput> KernelOutputExt for SerialKernelOutput<T> {}
