/* use core::{bstr::ByteStr, fmt};

use bstr::BStr;
use unicode_normalization::UnicodeNormalization;
use zerOS_macro_utils::max;

pub trait KernelTextInput
{
	fn read_byte(&mut self) -> u8;

	fn read_bytes(&mut self, bytes: &mut [u8], max: Option<usize>)
	{
		if let Some(max_written) = max
		{
			let absmax = bytes.len();
			for i in 0..(max!(max_written, absmax))
			{
				bytes[i] = self.read_byte();
			}
		}
		else
		{
			for b in bytes
			{
				*b = self.read_byte();
			}
		}
	}
}

pub trait KernelTextOutput
{
	/// Flush the IO stream, if needed
	fn flush(&mut self) -> fmt::Result;

	/// Whether or not it supports things akin to terminal color escape codes
	fn supports_ansi_escape_codes(&self) -> bool;

	fn write_byte(&mut self, byte: u8) -> fmt::Result;

	unsafe fn write_utf8_bytes_unchecked(&mut self, bytes: &[u8]) -> fmt::Result
	{
		let mut buf = [0; 4];
		for b in unsafe { str::from_utf8_unchecked(bytes).nfc() }
			.map(|ch| ch.encode_utf8(&mut buf))
			.flat_map(|s| s.bytes())
		{
			self.write_byte(*b)?;
		}
		Ok(())
	}
}

impl fmt::Write for dyn KernelTextOutput
{
	fn write_str(&mut self, s: &str) -> fmt::Result
	{
		todo!()
	}

	fn write_char(&mut self, c: char) -> fmt::Result
	{
		todo!()
	}

	fn write_fmt(&mut self, args: fmt::Arguments<'_>) -> fmt::Result
	{
		todo!()
	}
}

pub trait KernelTextIO = KernelTextInput + KernelTextOutput;

pub trait KernelTextInputExt: KernelTextInput
{
	fn read_bytes_while(
		&mut self,
		bytes: &mut [u8],
		mut predicate: impl FnMut(&u8) -> bool
	) -> usize
	{
		let (mut i, absmax): (usize, usize) = (0, bytes.len());
		while i < absmax
		{
			let buf = self.read_byte();
			if !predicate(&buf)
			{
				return i;
			}
			bytes[i] = buf;
			i += 1;
		}
		i
	}
	fn read_bytes_until(
		&mut self,
		bytes: &mut [u8],
		mut predicate: impl FnMut(&u8) -> bool
	) -> usize
	{
		self.read_bytes_while(bytes, |val| !predicate(val))
	}
}

pub trait KernelTextOutputExt: KernelTextOutput {}

pub trait KernelTextIOExt = KernelTextInputExt + KernelTextOutputExt;
 */