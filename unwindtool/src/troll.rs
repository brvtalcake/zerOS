use std::{borrow::Cow, collections::HashMap};

use anyhow::{Result, anyhow};
use fallible_iterator::FallibleIterator;
use gimli::{BaseAddresses, CfiEntriesIter, CieOrFde, EhFrame, EhFrameOffset, Register, UnwindContext, UnwindSection, UnwindTableRow};
use object::{
	Endian,
	Endianness,
	Object,
	ObjectSection,
	ReadRef,
	elf::SHF_EXECINSTR,
	read::elf::{ElfFile64, SectionHeader}
};

use crate::SupportedArch;

pub type Troll64PCRange = (u64, u64);
pub struct TrollData<'data>
{
	pub regs:            TrollRegisterSet,
	pub source_location: TrollLocation<'data>
}

pub struct TrollLocation<'data>
{
	pub file:   Cow<'data, str>,
	pub line:   usize,
	pub column: usize
}

pub struct Troll64<'data, 'file, R: ReadRef<'data>>
{
	elf:    &'file ElfFile64<'data, Endianness, R>,
	pc_map: HashMap<Troll64PCRange, TrollData<'data>>,
	arch: SupportedArch,
}

impl<'data, 'file, R: ReadRef<'data>> Troll64<'data, 'file, R>
{
	fn map_from_elf(
		arch: SupportedArch,
		elf: &'file ElfFile64<'data, Endianness, R>
	) -> Result<HashMap<Troll64PCRange, TrollData<'data>>>
	{
		let eh_frame_hdr = elf
			.section_by_name(".eh_frame_hdr")
			.ok_or(anyhow!("couldn't find \".eh_frame_hdr\" section"))?;
		let eh_frame = elf
			.section_by_name(".eh_frame")
			.ok_or(anyhow!("couldn't find \".eh_frame\" section"))?;
		let base_addresses = BaseAddresses::default()
			.set_eh_frame_hdr(eh_frame_hdr.address())
			.set_eh_frame(eh_frame.address());
		let uncompressed_eh_frame = eh_frame.uncompressed_data()?;
		let unwind_info = EhFrame::new(
			&uncompressed_eh_frame,
			if elf.endian().is_little_endian()
			{
				gimli::RunTimeEndian::Little
			}
			else
			{
				gimli::RunTimeEndian::Big
			}
		);
		// let mut executable_sections = vec![];
		let mut executable_size = 0;
		for section in elf.sections()
		{
			if section.elf_section_header().sh_flags(elf.endian()) & (SHF_EXECINSTR as u64)
				== (SHF_EXECINSTR as _)
			{
				// executable_sections.push(&section);
				executable_size += section.size() as usize;
			}
		}
		let mut map = HashMap::with_capacity((executable_size / 10) + 1);
		let mut entries = unwind_info.entries(&base_addresses);
		let mut cies: HashMap<EhFrameOffset, _> = HashMap::new();
        let mut ctx = UnwindContext::new();
		while let Some(entry) = entries.next()?
		{
			// TODO: when we hit a CIE, save it somewhere to somehow re-use it when hitting
			// an FDE (?)
			match entry
			{
				CieOrFde::Fde(partial_fde) =>
				{
					let fde = partial_fde.parse(|sect, base_addrs, offset| {
						cies.get(&offset).cloned().map_or_else(
							|| {
								sect.cie_from_offset(base_addrs, offset).inspect(|cie| {
									cies.insert(offset, cie.clone());
								})
							},
							Ok
						)
					})?;
					let mut rows = fde.rows(&unwind_info, &base_addresses, &mut ctx)?;
                    while let Some(row) = rows.next_row()?
                    {
                        let range = (row.start_address(), row.end_address());
						let regs = get_regs(arch, row);
                        todo!("do something with row");
                    }
				},
				CieOrFde::Cie(cie) =>
				{
					cies.insert(cie.offset().into(), cie);
				}
			}
		}
		Ok(map)
	}

	pub fn new(arch: SupportedArch, elf: &'file ElfFile64<'data, Endianness, R>) -> Result<Self>
	{
		Ok(Self {
			elf,
			pc_map: Self::map_from_elf(arch, elf)?,
			arch
		})
	}
}

fn get_regs(arch: SupportedArch, row: &UnwindTableRow<usize>) -> TrollRegisterSet
{
	todo!();
	match arch
	{
		SupportedArch::Amd64 => {
			/* let ra = row.register(gimli::X86_64::RA); */
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrollRegisterSet
{
	Amd64
	{
		rsp: u64,
		rbp: u64,
		ret: u64 // `$RA` in DWARF parlance
	},
	X86,
	AArch64,
	Arm32,
	Riscv32,
	Riscv64,
	PowerPC32,
	PowerPC64,
	Sparc32,
	Sparc64,
	Mips32,
	Mips64,
	Avr32,
	LoongArch64,
	ZArch
}
