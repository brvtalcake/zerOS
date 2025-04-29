use crate::kernel::linker::LinkerSym;

unsafe extern "C" {
	unsafe static __ctor_init_array_start: LinkerSym;
	unsafe static __ctor_init_array_end: LinkerSym;
}

pub type Ctor = unsafe extern "C" fn();

const CTOR_SIZE: usize = size_of::<Ctor>();

fn ctor_count(start: LinkerSym, end: LinkerSym) -> usize
{
	let usize_start = start as usize;
	let usize_end = end as usize;
	(usize_end - usize_start) / CTOR_SIZE
}

pub struct CtorIter
{
	start: LinkerSym,
	cur:   usize,
	count: usize
}

#[overloadf::overload]
impl CtorIter
{
	pub const fn new(start: LinkerSym, end: LinkerSym) -> Self
	{
		Self {
			start,
			cur: 0,
			count: unsafe { ctor_count(start, end) }
		}
	}

	pub const fn new() -> Self
	{
		Self {
			start: unsafe { __ctor_init_array_start },
			cur:   0,
			count: unsafe { ctor_count(__ctor_init_array_start, __ctor_init_array_end) }
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
			Some(unsafe {
				core::mem::transmute::<_, Ctor>(
					((self.start as usize) + (self.cur * CTOR_SIZE)) as LinkerSym
				)
			})
		}
	}
}

impl Iterator for CtorIter
{
	type Item = Ctor;

	fn next(&mut self) -> Option<Self::Item>
	{
		self.count += 1;
		self.get_at_cur()
	}
}
