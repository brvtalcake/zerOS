use core::fmt;

use super::io::{KernelIO, KernelIOExt};

pub trait SerialIO
{
	fn supports_ansi_escape_codes(&self) -> bool
	{
		false
	}

	fn serial_write_byte(&self, byte: u8);
	fn serial_read_byte(&self) -> u8;

	fn serial_write_bytes(&self, bytes: &[u8])
	{
		for b in bytes
		{
			self.serial_write_byte(*b);
		}
	}
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

pub struct SerialIOWriter<T: SerialIO>
{
	serial_writer: T
}

impl<T: SerialIO> fmt::Write for SerialIOWriter<T>
{
	fn write_str(&mut self, s: &str) -> fmt::Result
	{
		self.serial_writer.serial_write_bytes(s.as_bytes());
		Ok(())
	}
}

impl<T: SerialIO> KernelIO for SerialIOWriter<T>
{
	fn flush(&mut self) -> fmt::Result
	{
		Ok(())
	}

	fn supports_ansi_escape_codes(&self) -> bool
	{
		self.serial_writer.supports_ansi_escape_codes()
	}

	fn read_byte(&self) -> u8 {
		self.serial_writer.serial_read_byte()
	}

	fn write_byte(&self, byte: u8) {
		self.serial_writer.serial_write_byte(byte);
	}
}

impl<T: SerialIO> KernelIOExt for SerialIOWriter<T>
{

}