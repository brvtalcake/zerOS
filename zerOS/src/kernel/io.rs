use core::fmt;

pub trait KernelIO: fmt::Write
{
	/// Flush the IO stream, if needed
	fn flush(&mut self) -> fmt::Result;

	/// Whether or not it supports things akin to terminal color escape codes
	fn supports_ansi_escape_codes(&self) -> bool;

	fn write_byte(&self, byte: u8);
	fn read_byte(&self) -> u8;

	fn write_bytes(&self, bytes: &[u8])
	{
		for b in bytes
		{
			self.write_byte(*b);
		}
	}
	fn read_bytes(&self, bytes: &mut [u8], max: Option<usize>)
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

pub trait KernelIOExt: KernelIO
{
    fn read_bytes_while(&self, bytes: &mut [u8], mut predicate: impl FnMut(&u8) -> bool) -> usize
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
	fn read_bytes_until(&self, bytes: &mut [u8], mut predicate: impl FnMut(&u8) -> bool) -> usize
	{
		self.read_bytes_while(bytes, |val| !predicate(val))
	}
}
