use core::{
	arch::asm,
	mem::{align_of, size_of}
};

use num_traits::AsPrimitive;
use zerocopy::{FromBytes, IntoBytes};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GDTDescriptor
{
	pub size:   u16,
	pub offset: u64
}
static_assert!(size_of::<GDTDescriptor>() == 10);

bitfield! {
	#[repr(C, packed)]
	#[derive(Debug, Clone, Copy)]
	#[derive(IntoBytes, FromBytes)]
	#[provide(default = true)]
	#[provide(ctor = true)]
	#[provide(as_ref = true)]
	#[provide(as_mut = true)]
	#[check(equal_to = 16)]
	pub struct GDTSelector -> 16
	{
		pub _ rpl  : 2;
		pub _ table: 1;
		pub _ index: 13;
	}
}
static_assert!(size_of::<GDTSelector>() == 2);
static_assert!(align_of::<GDTSelector>() == 1);

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GDTSegmentRegisters
{
	pub cs: GDTSelector,
	pub ds: GDTSelector,
	pub es: GDTSelector,
	pub fs: GDTSelector,
	pub gs: GDTSelector,
	pub ss: GDTSelector
}

static_assert!(size_of::<GDTSegmentRegisters>() == 6 * size_of::<GDTSelector>());

bitfield! {
	#[repr(C, packed)]
	#[derive(Debug, Clone, Copy)]
	#[derive(IntoBytes, FromBytes)]
	#[provide(default = true)]
	#[provide(ctor = true)]
	#[provide(as_ref = true)]
	#[provide(as_mut = true)]
	#[check(equal_to = 64)]
	pub struct GDTNormalSegmentDescriptor -> 64
	{
		pub usize limit_low: 16;
		pub usize base_low : 24;
		union {
			pub _ access: 8;
			struct {
				pub _ accessed: 1;  //< Is the segment accessed ?
				pub _ rw_bit: 1;    //< Additional read or write permissions
				pub _ dc_bit: 1;    //< Grows down or up ? (for data segments). Conforming ? (for code segments)
				pub _ exec_bit: 1;  //< Code segment if 1, data segment if 0
				pub _ desc_type: 1; //< Descriptor type
				pub _ priv_lvl: 2;  //< Privilege level
				pub _ present: 1;   //< Present bit
			};
		};
		pub usize limit_hi: 4;
		union {
			pub _ flags: 4;
			struct {
				pub _ _reserved: 1;
				pub _ long_mode: 1;
				pub _ size: 1;
				pub _ granularity: 1;
			};
		};
		pub usize base_hi: 8;
	}
}

impl GDTNormalSegmentDescriptor
{
	pub fn get_base(&self) -> usize
	{
		let base_low = self.get_base_low();
		let base_hi = self.get_base_hi();
		(base_low as usize) | ((base_hi as usize) << 24)
	}

	pub fn set_base(&mut self, value: usize)
	{
		self.set_base_low((value & 0xffffff_usize).as_());
		self.set_base_hi(((value >> 24) & 0xff_usize).as_());
	}

	pub fn with_base(&mut self, value: usize) -> &mut Self
	{
		self.set_base(value);
		self
	}

	pub fn get_limit(&self) -> usize
	{
		let limit_low = self.get_limit_low();
		let limit_hi = self.get_limit_hi();
		(limit_low as usize) | ((limit_hi as usize) << 16)
	}

	pub fn set_limit(&mut self, value: usize)
	{
		self.set_limit_low((value & 0xffff_usize).as_());
		self.set_limit_hi(((value >> 16) & 0xf_usize).as_());
	}

	pub fn with_limit(&mut self, value: usize) -> &mut Self
	{
		self.set_limit(value);
		self
	}
}

bitfield! {
	#[repr(C, packed)]
	#[derive(Debug, Clone, Copy)]
	#[derive(IntoBytes, FromBytes)]
	#[provide(default = true)]
	#[provide(ctor = true)]
	#[provide(as_ref = true)]
	#[provide(as_mut = true)]
	#[check(equal_to = 128)]
	pub struct GDTSystemSegmentDescriptor -> 128
	{
		pub usize limit_low: 16;
		pub usize base_low : 24;
		union {
			pub _ access: 8;
			struct {
				pub _ sys_type: 4;  //< Type of system segment
				pub _ desc_type: 1; //< Descriptor type
				pub _ priv_lvl: 2;  //< Privilege level
				pub _ present: 1;   //< Present bit
			};
		};
		pub usize limit_hi: 4;
		union {
			pub _ flags: 4;
			struct {
				pub _ available: 1;
				pub _ _unused: 2;
				pub _ granularity: 1;
			};
		};
		pub usize base_hi: 40;
		pub _ _reserved: 32;
	}
}

impl GDTSystemSegmentDescriptor
{
	pub fn get_base(&self) -> usize
	{
		let base_low = self.get_base_low();
		let base_hi = self.get_base_hi();
		(base_low as usize) | ((base_hi as usize) << 24)
	}

	pub fn set_base(&mut self, value: usize)
	{
		self.set_base_low((value & 0xffffff_usize).as_());
		self.set_base_hi(((value >> 24) & 0xffffffffff_usize).as_());
	}

	pub fn with_base(&mut self, value: usize) -> &mut Self
	{
		self.set_base(value);
		self
	}

	pub fn get_limit(&self) -> usize
	{
		let limit_low = self.get_limit_low();
		let limit_hi = self.get_limit_hi();
		(limit_low as usize) | ((limit_hi as usize) << 16)
	}

	pub fn set_limit(&mut self, value: usize)
	{
		self.set_limit_low((value & 0xffff_usize).as_());
		self.set_limit_hi(((value >> 16) & 0xf_usize).as_());
	}

	pub fn with_limit(&mut self, value: usize) -> &mut Self
	{
		self.set_limit(value);
		self
	}
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub union GDTEntry
{
	pub norm: [GDTNormalSegmentDescriptor; 2],
	pub sys:  GDTSystemSegmentDescriptor
}

impl Default for GDTEntry
{
	fn default() -> Self
	{
		Self {
			norm: <[GDTNormalSegmentDescriptor; 2]>::default()
		}
	}
}

#[derive(Clone, Copy)]
pub struct GDT
{
	pub entries: [GDTEntry; entry_index!(MAX) / 2]
}

pub macro entry_index
{
    (NULL) => { 0 },

    (KERNEL32_CS) => { 1 },
    (KERNEL64_CS) => { 2 },
    (KERNEL_DS) => { 3 },

    (USER32_CS) => { 4 },
    (USER_DS) => { 5 },
    (USER64_CS) => { 6 },

    (TSS) => { 8 },

    (KERNEL_TLS) => { 10 },
    (USER_TLS) => { 11 },

    (MAX) => { 16 },

    (UNASSIGNED) => { 7, 12, 13, 14, 15 },
}

pub macro entry_make
{
    (NULL) => {
        GDT::make_null_segment()
    },

    (KERNEL32_CS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, true, 0, true,
            false, true, true
        )
    },
    (KERNEL64_CS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, true, 0, true,
            true, false, true
        )
    },
    (KERNEL_DS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, false, 0, true,
            false, true, true
        )
    },

    (USER32_CS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, true, 3, true,
            false, true, true
        )
    },
    (USER_DS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, false, 3, true,
            false, true, true
        )
    },
    (USER64_CS) => {
        GDT::make_normal_segment(
            0_usize, usize::MAX,
            true, true, false, true, 3, true,
            true, false, true
        )
    },

    (TSS) => {
        GDT::make_system_segment(
            0_usize, 0_usize,
            9, 0, true,
            true, true
        )
    },

    (KERNEL_TLS) => { entry_make!(KERNEL_DS) },
    (USER_TLS) => { entry_make!(USER_DS) },

    (MAX) => { entry_make!(NULL) },

    (UNASSIGNED) => { entry_make!(NULL) },
}

impl GDT
{
	fn make_null_segment_private<T: Default>() -> T
	{
		T::default()
	}

	pub fn make_null_segment() -> GDTNormalSegmentDescriptor
	{
		Self::make_null_segment_private()
	}

	#[allow(clippy::too_many_arguments)]
	pub fn make_normal_segment(
		base: usize,
		limit: usize,
		accessed: bool,
		rw_bit: bool,
		dc_bit: bool,
		exec_bit: bool,
		priv_lvl: u8,
		present: bool,
		long_mode: bool,
		size: bool,
		granularity: bool
	) -> GDTNormalSegmentDescriptor
	{
		*Self::make_null_segment_private::<GDTNormalSegmentDescriptor>()
			.with_base(base.as_())
			.with_limit(limit.as_())
			.with_accessed(accessed.as_())
			.with_rw_bit(rw_bit.as_())
			.with_dc_bit(dc_bit.as_())
			.with_exec_bit(exec_bit.as_())
			.with_desc_type(1.as_())
			.with_priv_lvl(priv_lvl.as_())
			.with_present(present.as_())
			.with_long_mode(long_mode.as_())
			.with_size(size.as_())
			.with_granularity(granularity.as_())
	}

	pub fn make_system_segment(
		base: usize,
		limit: usize,
		sys_type: u8,
		priv_lvl: u8,
		present: bool,
		available: bool,
		granularity: bool
	) -> GDTSystemSegmentDescriptor
	{
		*Self::make_null_segment_private::<GDTSystemSegmentDescriptor>()
			.with_base(base.as_())
			.with_limit(limit.as_())
			.with_sys_type(sys_type.as_())
			.with_desc_type(0.as_())
			.with_priv_lvl(priv_lvl.as_())
			.with_present(present.as_())
			.with_available(available.as_())
			.with_granularity(granularity.as_())
	}

	/// # SAFETY
	///
	/// If not calling with `Default::default()` generated
	/// GDT, be extremely cautious with the offsets of
	/// `GDTSystemSegmentDescriptor`s
	pub unsafe fn set(&self)
	{
		let gdt_desc = GDTDescriptor {
			// (zerOS_GDT_ENTRY_INDEX_MAX * sizeof(struct zerOS_gdt_normal_segment_descriptor)) - 1
			size:   ((entry_index!(MAX) * size_of::<GDTNormalSegmentDescriptor>()) - 1).as_(),
			offset: self.entries.as_ptr() as u64
		};
		let gdt_desc_ptr: *const GDTDescriptor = &gdt_desc;
		let gdt_regs = GDTSegmentRegisters {
			cs: *GDTSelector::default().with_index(entry_index!(KERNEL64_CS)),
			ds: *GDTSelector::default().with_index(entry_index!(KERNEL_DS)),
			es: *GDTSelector::default().with_index(entry_index!(KERNEL_DS)),
			fs: *GDTSelector::default().with_index(entry_index!(KERNEL_DS)),
			gs: *GDTSelector::default().with_index(entry_index!(KERNEL_DS)),
			ss: *GDTSelector::default().with_index(entry_index!(KERNEL_DS))
		};
		let gdt_regs_ptr: *const GDTSegmentRegisters = &gdt_regs;
		crate::arch::target::cpu::irq::disable();
		unsafe {
			asm! {
				"lgdt 0({gdt_desc_ptr_reg})",
				"xor %rax, %rax",
				"movw 0({gdt_regs_ptr_reg}), %ax",
				"pushq %rax",
				"xor %rax, %rax",
				"leaq 2f(%rip), %rax",
				"pushq %rax",
				"lretq",
			"2:",
				"movw  2({gdt_regs_ptr_reg}), %ds",
				"movw  4({gdt_regs_ptr_reg}), %es",
				"movw  6({gdt_regs_ptr_reg}), %fs",
				"movw  8({gdt_regs_ptr_reg}), %gs",
				"movw 10({gdt_regs_ptr_reg}), %ss",
				// TODO: which registers really end up being clobbered ?
				out("rax") _,
				gdt_desc_ptr_reg = inout(reg) gdt_desc_ptr => _,
				gdt_regs_ptr_reg = inout(reg) gdt_regs_ptr => _,
				options(att_syntax)
			};
		}
		crate::arch::target::cpu::irq::enable();
	}
}

macro_rules! entry_ids
{
    (@skip_symbolic $($tokens:tt)*) => {
        callback!(
            $($tokens)*(
                NULL,
                KERNEL32_CS, KERNEL64_CS, KERNEL_DS,
                USER32_CS, USER_DS, USER64_CS,
                TSS,
                KERNEL_TLS, USER_TLS
            )
        )
    };
    ($($tokens:tt)*) => {
        callback!(
            $($tokens)*(
                NULL,
                KERNEL32_CS, KERNEL64_CS, KERNEL_DS,
                USER32_CS, USER_DS, USER64_CS,
                TSS,
                KERNEL_TLS, USER_TLS,
                MAX,
                UNASSIGNED
            )
        )
    };
}

impl Default for GDT
{
	fn default() -> Self
	{
		let mut this = Self {
			entries: <[GDTEntry; entry_index!(MAX) / 2]>::default()
		};
		macro_rules! if_normal_segment {
			(TSS, $iftrue:block, $iffalse:block) => {
				$iffalse
			};
			($other:ident, $iftrue:block, $iffalse:block) => {
				$iftrue
			};
		}
		macro_rules! def_entry {
			($name:ident) => {{
				let (ind, ent) = (entry_index!($name), entry_make!($name));
				if_normal_segment!(
					$name,
					{
						let (div, modulus) = (ind / 2, ind % 2);
						unsafe {
							this.entries[div].norm[modulus] = ent;
						}
					},
					{
						let div = ind / 2;
						this.entries[div].sys = ent;
					}
				);
			};};
		}
		entry_ids!(@skip_symbolic @foreach @delim(;) def_entry);
		this
	}
}
