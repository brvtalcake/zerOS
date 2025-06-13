//! # Acknoledgments
//! Code initially copy-pasted/adapted from https://lesenechal.fr/en/linux/unwinding-the-stack-the-hard-way
//!
//! # Future direction
//! As an idea for future work, try to *REALLY* unwind the stack, akin to what [Theseus OS](https://github.com/theseus-os/Theseus/blob/1fbfe567075a65ed749b6680db1aeb538819c70c/kernel/unwind/src/lib.rs#L626) does.

use alloc::string::String;

use cfg_if::cfg_if;
use corosensei::{Coroutine, CoroutineResult};
use gimli::{
	CfaRule,
	DW_AT_ranges,
	Register,
	RegisterRule,
	Section,
	UnitType,
	UnwindContext,
	UnwindSection
};
use num::traits::AsPrimitive;

cfg_if! {
	if #[cfg(target_arch = "x86_64")]
	{
		mod amd64;
		pub use self::amd64::{
			EhInfo,
			unwind_get_registers as read_registers,
			RegisterSet
		};
	}
	else if #[cfg(target_arch = "x86")]
	{
		mod x86;
		pub use self::x86::{
			EhInfo,
			unwind_get_registers as read_registers,
			RegisterSet
		};
	}
	else
	{
		compile_error!("TODO: implement stack unwinding/backtraces for this target !");
	}
}

// TODO: separate the .eh_frame_hdr/.eh_frame sections and put them in a
// separate file to be loaded by Limine as a kernel module (?)
pub struct Unwinder
{
	/// The call frame information.
	eh_info: EhInfo,

	/// A `UnwindContext` needed by Gimli for optimizations.
	unwind_ctx: UnwindContext<usize>,

	/// The current values of registers. These values are updated as we restore
	/// register values.
	regs: RegisterSet,

	/// The current CFA address.
	cfa: u64,

	/// Is it the first iteration?
	is_first: bool
}

impl Unwinder
{
	pub fn new(eh_info: EhInfo, register_set: RegisterSet) -> Self
	{
		Self {
			eh_info,
			unwind_ctx: UnwindContext::new(),
			regs: register_set,
			cfa: 0,
			is_first: true
		}
	}

	pub fn next(&mut self) -> Result<Option<CallFrame>, UnwinderError>
	{
		let pc = self.regs.get_pc().ok_or(UnwinderError::NoPcRegister)?;

		if self.is_first
		{
			self.is_first = false;
			return Ok(Some(CallFrame { pc: pc.as_() }));
		}

		let row = self
			.eh_info
			.hdr
			.table()
			.unwrap()
			.unwind_info_for_address(
				&self.eh_info.eh_frame,
				&self.eh_info.base_addrs,
				&mut self.unwind_ctx,
				pc,
				|section, bases, offset| section.cie_from_offset(bases, offset)
			)
			.map_err(|_| UnwinderError::NoUnwindInfo)?;

		match row.cfa()
		{
			CfaRule::RegisterAndOffset { register, offset } =>
			{
				let reg_val = self
					.regs
					.get(*register)
					.ok_or(UnwinderError::CfaRuleUnknownRegister(*register))?;
				self.cfa = (reg_val as i64 + offset) as u64;
			},
			_ => return Err(UnwinderError::UnsupportedCfaRule)
		}

		for reg in RegisterSet::iter()
		{
			match row.register(reg)
			{
				RegisterRule::Undefined => self.regs.undef(reg),
				RegisterRule::SameValue => (),
				RegisterRule::Offset(offset) =>
				{
					let ptr = (self.cfa as i64 + offset) as u64 as *const usize;
					self.regs.set(reg, unsafe { ptr.read() } as u64)?;
				},
				_ => return Err(UnwinderError::UnimplementedRegisterRule)
			}
		}

		let pc = self.regs.get_ret().ok_or(UnwinderError::NoReturnAddr)? - 1;
		self.regs.set_pc(pc);
		self.regs.set_stack_ptr(self.cfa);

		let units_iter = self.eh_info.debug_info.units();
		while let Some(unit) = units_iter.next().map_err(|_| UnwinderError::NoSourceInfo)?
		{
			if unit.type_() == UnitType::Compilation
			{
				if let Some(abbrev) = unit
					.abbreviations(&self.eh_info.debug_abbrev)
					.map_err(|_| UnwinderError::NoSourceInfo)?
					.get(pc)
				{
					if let Some(found_attr) = abbrev
						.attributes()
						.iter()
						.find(|attr| attr.name() == DW_AT_ranges)
					{
						todo!()
					}
				}
			}
		}

		Ok(Some(CallFrame { pc: pc.as_() }))
	}
}

#[derive(Debug)]
pub struct CallFrame
{
	pub pc:       usize,
	pub function: String,
	pub line:     usize,
	pub column:   usize
}

impl CallFrame {}

#[derive(Debug, strum::AsRefStr)]
#[strum(serialize_all = "kebab-case")]
pub enum UnwinderError
{
	UnexpectedRegister(Register),
	UnsupportedCfaRule,
	UnimplementedRegisterRule,
	CfaRuleUnknownRegister(Register),
	NoUnwindInfo,
	NoSourceInfo,
	NoPcRegister,
	NoReturnAddr
}
