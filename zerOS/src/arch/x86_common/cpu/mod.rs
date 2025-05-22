use cfg_if::cfg_if;
use macro_utils::{CaseKind, MultiCaseStaticString};
use overloadable::overloadable;

cfg_if! {
	if #[cfg(target_arch = "x86_64")] {
		pub use core::arch::x86_64::CpuidResult;
		use core::arch::x86_64::__cpuid_count;
	} else if #[cfg(target_arch = "x86")] {
		pub use core::arch::x86::CpuidResult;
		use core::arch::x86::__cpuid_count;
	} else {
		compile_error!("should be unreachable !");
	}
}

pub mod io;
pub mod irq;
pub mod misc;
mod registers;

pub mod ctlregs
{
	pub use super::registers::control::*;
}

pub mod xctlregs
{
	pub use super::registers::control_extended::*;
}

pub mod dbgregs
{
	pub use super::registers::debug::*;
}

pub mod msr
{
	pub use super::registers::msr::*;
}

pub use raw_cpuid as features;

#[gen_variant_names]
#[repr(usize)]
#[derive(Clone, Copy)]
pub enum GeneralPurposeRegisterId
{
	RAX = 0, // EAX   AX    AH  AL    Accumulator
	RBX,     // EBX   BX    BH  BL    Base
	RCX,     // ECX   CX    CH  CL    Counter
	RDX,     // EDX   DX    DH  DL    Data (commonly extends the A register)
	RSI,     // ESI   SI    NA  SIL   Source index for string operations
	RDI,     // EDI   DI    NA  DIL   Destination index for string operations
	RSP,     // ESP   SP    NA  SPL   Stack Pointer
	RBP,     // EBP   BP    NA  BPL   Base Pointer (meant for stack frames)
	R8,      // R8D   R8W   NA  R8B   General purpose
	R9,      // R9D   R9W   NA  R9B   General purpose
	R10,     // R10D  R10W  NA  R10B  General purpose
	R11,     // R11D  R11W  NA  R11B  General purpose
	R12,     // R12D  R12W  NA  R12B  General purpose
	R13,     // R13D  R13W  NA  R13B  General purpose
	R14,     // R14D  R14W  NA  R14B  General purpose
	R15      // R15D  R15W  NA  R15B  General purpose
}

impl GeneralPurposeRegisterId
{
	const BITS16_MNEMONICS: [&'static str; 16] = [
		"AX", "BX", "CX", "DX", "SI", "DI", "SP", "BP", "R8W", "R9W", "R10W", "R11W", "R12W",
		"R13W", "R14W", "R15W"
	];
	const BITS32_MNEMONICS: [&'static str; 16] = [
		"EAX", "EBX", "ECX", "EDX", "ESI", "EDI", "ESP", "EBP", "R8D", "R9D", "R10D", "R11D",
		"R12D", "R13D", "R14D", "R15D"
	];
	const BITS8HI_MNEMONICS: [&'static str; 16] = [
		"AH", "BH", "CH", "DH", "", "", "", "", "", "", "", "", "", "", "", ""
	];
	const BITS8LO_MNEMONICS: [&'static str; 16] = [
		"AL", "BL", "CL", "DL", "SIL", "DIL", "SPL", "BPL", "R8B", "R9B", "R10B", "R11B", "R12B",
		"R13B", "R14B", "R15B"
	];

	pub const unsafe fn bits64_mnemonic(&self) -> &'static str
	{
		self.variant_name(CaseKind::LowerCase)
	}

	pub const unsafe fn bits32_mnemonic(&self) -> &'static str
	{
		Self::BITS32_MNEMONICS[*self as usize]
	}

	pub const unsafe fn bits16_mnemonic(&self) -> &'static str
	{
		Self::BITS16_MNEMONICS[*self as usize]
	}

	pub const unsafe fn bits8hi_mnemonic(&self) -> &'static str
	{
		Self::BITS8HI_MNEMONICS[*self as usize]
	}

	pub const unsafe fn bits8lo_mnemonic(&self) -> &'static str
	{
		Self::BITS8LO_MNEMONICS[*self as usize]
	}
}

#[derive(Clone, Copy)]
pub enum GeneralPurposeRegisterSize
{
	Bits64,
	Bits32,
	Bits16,
	Bits8Hi,
	Bits8Lo
}

pub fn is_valid_gpreg(id: GeneralPurposeRegisterId, size: GeneralPurposeRegisterSize) -> bool
{
	match (id, size)
	{
		(
			_,
			GeneralPurposeRegisterSize::Bits64
			| GeneralPurposeRegisterSize::Bits32
			| GeneralPurposeRegisterSize::Bits16
			| GeneralPurposeRegisterSize::Bits8Lo
		)
		| (
			GeneralPurposeRegisterId::RAX
			| GeneralPurposeRegisterId::RBX
			| GeneralPurposeRegisterId::RCX
			| GeneralPurposeRegisterId::RDX,
			_
		) => true,
		_ => false
	}
}

pub fn gpreg_mnemonic(
	id: GeneralPurposeRegisterId,
	size: GeneralPurposeRegisterSize
) -> Option<&'static str>
{
	if !is_valid_gpreg(id, size)
	{
		return None;
	}

	let retstr = match size
	{
		GeneralPurposeRegisterSize::Bits64 =>
		unsafe { id.bits64_mnemonic() },
		GeneralPurposeRegisterSize::Bits32 =>
		unsafe { id.bits32_mnemonic() },
		GeneralPurposeRegisterSize::Bits16 =>
		unsafe { id.bits16_mnemonic() },
		GeneralPurposeRegisterSize::Bits8Hi =>
		unsafe { id.bits8hi_mnemonic() },
		GeneralPurposeRegisterSize::Bits8Lo =>
		unsafe { id.bits8lo_mnemonic() }
	};
	Some(retstr)
}

pub enum Register
{
	ModelSpecific
	{
		which: u32
	},
	GeneralPurpose
	{
		id:      GeneralPurposeRegisterId,
		op_size: GeneralPurposeRegisterSize
	},
	Debug
	{
		which: u8
	},
	Test
	{
		which: u8
	},
	Control
	{
		which: u8, extended: bool
	}
}

overloadable! {
	pub cpuid as

	fn(leaf: u32, subleaf: u32) -> CpuidResult
	{
		unsafe {
			__cpuid_count(leaf, subleaf)
		}
	},
	fn(leaf: u32) -> CpuidResult
	{
		unsafe {
			__cpuid_count(leaf, 0)
		}
	}
}
