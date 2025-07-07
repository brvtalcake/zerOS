use alloc::{boxed::Box, slice};

use gimli::{
	BaseAddresses,
	DebugAbbrev,
	DebugInfo,
	DebugLine,
	EhFrame,
	EhFrameHdr,
	EndianSlice,
	NativeEndian,
	ParsedEhFrameHdr,
	Register,
	X86_64
};
use num::traits::AsPrimitive;

use super::UnwinderError;
use crate::kernel::linker::map::{
	zerOS_eh_frame_hdr_size,
	zerOS_eh_frame_hdr_start,
	zerOS_eh_frame_size,
	zerOS_eh_frame_start,
	zerOS_text_start
};

pub struct EhInfo
{
	/// A set of base addresses used for relative addressing.
	pub(super) base_addrs: BaseAddresses,

	/// The parsed `.eh_frame_hdr` section.
	pub(super) hdr: Box<ParsedEhFrameHdr<EndianSlice<'static, NativeEndian>>>,

	/// The parsed `.eh_frame` containing the call frame information.
	pub(super) eh_frame: EhFrame<EndianSlice<'static, NativeEndian>>,

	/// Source location information
	pub(super) debug_line: DebugLine<EndianSlice<'static, NativeEndian>>,

	/// Debug info
	pub(super) debug_info: DebugInfo<EndianSlice<'static, NativeEndian>>,

	/// Debug Abbrev
	pub(super) debug_abbrev: DebugAbbrev<EndianSlice<'static, NativeEndian>>
}

impl EhInfo
{
	pub fn new() -> Self
	{
		let mut base_addrs = BaseAddresses::default();
		// We set the `.eh_frame_hdr`’s address in the set of base addresses,
		// this will typically be used to compute the `.eh_frame` pointer.
		base_addrs =
			base_addrs.set_eh_frame_hdr((&raw const zerOS_eh_frame_hdr_start).addr().as_());

		// The `.eh_frame_hdr` is parsed by Gimli.
		let hdr = Box::new(
			EhFrameHdr::new(
				unsafe {
					slice::from_raw_parts(
						&raw const zerOS_eh_frame_hdr_start,
						*zerOS_eh_frame_hdr_size
					)
				},
				NativeEndian
			)
			.parse(&base_addrs, 8)
			.unwrap()
		);

		// We then add the `.eh_frame` address for addresses relative to that
		// section.
		base_addrs = base_addrs.set_eh_frame((&raw const zerOS_eh_frame_start).addr().as_());

		// The `.eh_frame` section is then parsed.
		let eh_frame = EhFrame::new(
			unsafe { slice::from_raw_parts(&raw const zerOS_eh_frame_start, *zerOS_eh_frame_size) },
			NativeEndian
		);

		base_addrs = base_addrs.set_text((&raw const zerOS_text_start).addr().as_());

		let debug_info = DebugInfo::new(
			unsafe {
				slice::from_raw_parts(&raw const zerOS_debug_info_start, *zerOS_debug_info_size)
			},
			NativeEndian
		);

		let source_info = DebugLine::new(
			unsafe {
				slice::from_raw_parts(&raw const zerOS_debug_line_start, *zerOS_debug_line_size)
			},
			NativeEndian
		);

		let debug_abbrev = DebugAbbrev::new(
			unsafe {
				slice::from_raw_parts(
					&raw const zerOS_debug_abbrev_start,
					*zerOS_debug_abbrev_size
				)
			},
			NativeEndian
		);

		Self {
			base_addrs,
			hdr,
			eh_frame,
			debug_line: source_info,
			debug_info,
			debug_abbrev
		}
	}
}

#[derive(Debug, Default)]
pub struct RegisterSet
{
	pub rip: Option<u64>,
	pub rsp: Option<u64>,
	pub rbp: Option<u64>,
	pub ret: Option<u64>
}

pub macro unwind_get_registers() {{
	let ip;
	let sp;
	let bp;
	let ret;
	unsafe {
		// TODO: what is the modifier for 64bits register operands ?
		::core::arch::asm! {
			"mov %rip, {ip}",
			"mov %rsp, {sp}",
			"mov %rbp, {bp}",
			ip = out(reg) ip,
			sp = out(reg) sp,
			bp = out(reg) bp,
			options(att_syntax)
		};
		ret = crate::llvm::return_address(0).addr() as u64;
	}
	crate::unwinding::RegisterSet {
		rip: Some(ip),
		rsp: Some(sp),
		rbp: Some(bp),
		ret: Some(ret)
	}
}}

impl RegisterSet
{
	pub(super) fn get(&self, reg: Register) -> Option<u64>
	{
		match reg
		{
			X86_64::RSP => self.rsp,
			X86_64::RBP => self.rbp,
			X86_64::RA => self.ret,
			_ => None
		}
	}

	pub(super) fn set(&mut self, reg: Register, val: u64) -> Result<(), UnwinderError>
	{
		*match reg
		{
			X86_64::RSP => &mut self.rsp,
			X86_64::RBP => &mut self.rbp,
			X86_64::RA => &mut self.ret,
			_ => return Err(UnwinderError::UnexpectedRegister(reg))
		} = Some(val);

		Ok(())
	}

	pub(super) fn undef(&mut self, reg: Register)
	{
		*match reg
		{
			X86_64::RSP => &mut self.rsp,
			X86_64::RBP => &mut self.rbp,
			X86_64::RA => &mut self.ret,
			_ => return
		} = None;
	}

	pub(super) fn get_pc(&self) -> Option<u64>
	{
		self.rip
	}

	pub(super) fn set_pc(&mut self, val: u64)
	{
		self.rip = Some(val);
	}

	pub(super) fn get_ret(&self) -> Option<u64>
	{
		self.ret
	}

	pub(super) fn set_stack_ptr(&mut self, val: u64)
	{
		self.rsp = Some(val);
	}

	pub(super) fn iter() -> impl Iterator<Item = Register>
	{
		[X86_64::RSP, X86_64::RBP, X86_64::RA].into_iter()
	}
}
