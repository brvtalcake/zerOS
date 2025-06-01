use crate::{
	func_at,
	kernel::linker::{
		LinkerSym,
		map::{zerOS_ctors_init_array_end, zerOS_ctors_init_array_start}
	}
};

#[unsafe(link_section = ".ctors_init_array")]
#[used(linker)]
static _SECTION_PLACE_HOLDER: [LinkerSym; 0] = [];

unsafe extern "C" {
	unsafe static __ctor_init_array_start: LinkerSym;
	unsafe static __ctor_init_array_end: LinkerSym;
}

pub type Ctor = unsafe extern "C" fn();

const CTOR_SIZE: usize = size_of::<Ctor>();

fn ctor_count(start: *const LinkerSym, end: *const LinkerSym) -> usize
{
	let usize_start = start as usize;
	let usize_end = end as usize;
	(usize_end - usize_start) / CTOR_SIZE
}

pub struct CtorIter
{
	start: *const LinkerSym,
	cur:   usize,
	count: usize
}

#[overloadf::overload]
impl CtorIter
{
	pub const fn new(start: *const LinkerSym, end: *const LinkerSym) -> Self
	{
		Self {
			start,
			cur: 0,
			count: ctor_count(start, end)
		}
	}

	pub const fn new() -> Self
	{
		Self {
			start: &raw const zerOS_ctors_init_array_start,
			cur:   0,
			count: ctor_count(
				&raw const zerOS_ctors_init_array_start,
				&raw const zerOS_ctors_init_array_end
			)
		}
	}

	fn get_at_cur(&self) -> Option<<Self as Iterator>::Item>
	{
		if self.cur >= self.count
		{
			None
		}
		else
		{
			unsafe {
				let func = func_at!(
					*(self.start.byte_add(self.cur * CTOR_SIZE) as *const *const ()) as Ctor
				);
				Some(func)
			}
		}
	}
}

impl Iterator for CtorIter
{
	type Item = Ctor;

	fn next(&mut self) -> Option<Self::Item>
	{
		let res = self.get_at_cur();
		if res.is_some()
		{
			self.cur += 1;
		}
		res
	}
}
