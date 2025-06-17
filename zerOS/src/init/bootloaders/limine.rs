use alloc::format;
use core::fmt::Display;

use limine::{
	BaseRevision,
	firmware_type::FirmwareType,
	modules::{InternalModule, ModuleFlags},
	paging,
	request::{
		BootloaderInfoRequest,
		DateAtBootRequest,
		DeviceTreeBlobRequest,
		EfiMemoryMapRequest,
		EfiSystemTableRequest,
		ExecutableAddressRequest,
		ExecutableCmdlineRequest,
		FirmwareTypeRequest,
		FramebufferRequest,
		HhdmRequest,
		MemoryMapRequest,
		ModuleRequest,
		MpRequest,
		PagingModeRequest,
		RsdpRequest,
		SmbiosRequest,
		StackSizeRequest
	}
};
use num::traits::AsPrimitive;

use crate::{error, info, warn};

macro_rules! requests {
    {$($it:item)*} => {
        $(
            #[used]
            #[unsafe(link_section = ".requests")]
            $it
        )*
    };
}

#[rustfmt::skip]
const MP_REQUEST_FLAGS: limine::mp::RequestFlags
	= if cfg!(x86_alike) {
		limine::mp::RequestFlags::X2APIC
	} else {
		limine::mp::RequestFlags::empty()
	};

requests! {
	pub static BASE_REVISION: BaseRevision
		= BaseRevision::new();
	pub static BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest
		= BootloaderInfoRequest::new();
	pub static BOOT_TIME_REQUEST: DateAtBootRequest
		= DateAtBootRequest::new();
	pub static DTB_REQUEST: DeviceTreeBlobRequest
		= DeviceTreeBlobRequest::new();
	pub static EFI_MEMMAP_REQUEST: EfiMemoryMapRequest
		= EfiMemoryMapRequest::new();
	pub static EFI_SYSTBL_REQUEST: EfiSystemTableRequest
		= EfiSystemTableRequest::new();
	pub static FIRMWARE_TYPE_REQUEST: FirmwareTypeRequest
		= FirmwareTypeRequest::new();
	pub static FRAMEBUFFER_REQUEST: FramebufferRequest
		= FramebufferRequest::new();
	pub static HHDM_REQUEST: HhdmRequest
		= HhdmRequest::new();
	pub static KERNEL_ADDRESS_REQUEST: ExecutableAddressRequest
		= ExecutableAddressRequest::new();
	pub static KERNEL_CMDLINE_REQUEST: ExecutableCmdlineRequest
		= ExecutableCmdlineRequest::new();
	pub static MEMMAP_REQUEST: MemoryMapRequest
		= MemoryMapRequest::new();
	pub static MODULES_REQUEST: ModuleRequest
		= ModuleRequest::new()
			.with_internal_modules(&[
				&InternalModule::new()
					.with_path(c"/zerOS-boot-modules/debug-info.zko")
					.with_cmdline(c"")
					.with_flags(ModuleFlags::empty())]);
	pub static MP_REQUEST: MpRequest
		= MpRequest::new()
			.with_flags(MP_REQUEST_FLAGS);
	pub static PAGING_MODE_REQUEST: PagingModeRequest
		= PagingModeRequest::new()
			.with_min_mode(paging::Mode::FOUR_LEVEL)
			.with_max_mode(paging::Mode::FIVE_LEVEL);
	pub static RSDP_REQUEST: RsdpRequest
		= RsdpRequest::new();
	pub static SMBIOS_REQUEST: SmbiosRequest
		= SmbiosRequest::new();
	pub static STACK_SIZE_REQUEST: StackSizeRequest
		= StackSizeRequest::new()
			.with_size(1024 * 1024);
}

mod __markers
{
	use limine::request::{RequestsEndMarker, RequestsStartMarker};

	#[used]
	#[unsafe(link_section = ".requests_start_marker")]
	static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();
	#[used]
	#[unsafe(link_section = ".requests_end_marker")]
	static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();
}

macro implements_display($typ:ty)
{
	impls!($typ: Display)
}

static_assert!(
	implements_display!(bool) && implements_display!(u8) && !implements_display!(*const u8)
);

fn firmware_type_string(fwtype: FirmwareType) -> &'static str
{
	match fwtype
	{
		FirmwareType::X86_BIOS => "x86 BIOS",
		FirmwareType::UEFI_32 => "UEFI (32-bits)",
		FirmwareType::UEFI_64 => "UEFI (64-bits)",
		FirmwareType::SBI => "SBI",
		_ => "<unknown firmware>"
	}
}

fn verify_requests() -> bool
{
	macro_rules! good
	{
		($resp:expr, $field:ident) => {
			info!(
				event: "limine-boot",
				concat!(
					"\t",
					stringify!($field),
					": {:#?}"
				),
				$resp.$field()
			)
		};
		($resp:expr, $override:literal, $field:ident) => {
			info!(
				event: "limine-boot",
				concat!(
					"\t",
					$override,
					": {:#?}"
				),
				$resp.$field()
			)
		};
	}
	macro_rules! bad {
		($($tokens:tt)*) => {
			warn!(event: "limine-boot", "\t{}", format!($($tokens)*))
		};
	}
	macro_rules! very_bad {
		($($tokens:tt)*) => {
			error!(event: "limine-boot", "\t{}", format!($($tokens)*));
			return false
		};
	}
	macro_rules! expect {
		($resp:expr, $field:ident) => {
			good!($resp, $field);
		};
		($resp:expr, $field:ident, $value:expr $(, $other:expr)*) => {
			let _tmp = $resp.$field();
			if _tmp == ($value) $(|| _tmp == ($other))*
			{
				good!($resp, $field);
			}
			else
			{
				bad!(
					concat!(
						"\t",
						"expected `{}` for response field `",
						stringify!($field),
						"`, but got `{:#?}` instead"
					),
					stringify!($value $(|| $other)*),
					_tmp
				);
			}
		};
		($resp:expr, $override:literal, $field:ident, $value:expr $(, $other:expr)*) => {
			let _tmp = $resp.$field();
			if _tmp == ($value) $(|| _tmp == ($other))*
			{
				good!($resp, $override, $field);
			}
			else
			{
				bad!(
					concat!(
						"\t",
						"expected `{}` for `",
						$override,
						"`, but got `{:#?}` instead"
					),
					stringify!($value $(|| $other)*),
					_tmp
				);
			}
		};
	}

	macro_rules! verify {
		($((required: $required:expr))? $what:literal: $req:expr; { $( $(($override:literal))? $field:ident $( = $expected:expr $(, $other:expr)* )?; )*}) => {
			info!(
				event: "limine-boot",
				concat!(
					"verifying ",
					$what,
					"..."
				)
			);
			#[allow(unused_variables)]
			if let Some(resp) = $req.get_response()
			{
				$(
					expect!(
						resp,
						$($override, )?
						$field
						$(, $expected $(, $other)*)?
					);
				)*
			}
			else if true $(&& ($required as bool))? {
				very_bad!(
					concat!(
						"couldn't verify ",
						$what,
						" response: ",
						stringify!($req),
						" has no response field !"
					)
				);
			}
			else
			{
				info!(
					event: "limine-boot",
					concat!(
						$what,
						" not present, but this is expected, skipping..."
					)
				);
			}
		};
	}

	info!(event: "limine-boot", "start verifying Limine requests/responses");

	verify!(
		"bootloader info": BOOTLOADER_INFO_REQUEST;
		{
			name = "Limine";
			version;
		}
	);
	verify!(
		"date at boot": BOOT_TIME_REQUEST;
		{
			timestamp;
		}
	);
	verify!(
		(required: cfg!(not(x86_alike)))
		"device-tree blob": DTB_REQUEST;
		{
			dtb_ptr;
		}
	);
	verify!(
		// TODO: dump the memory map
		(required: false)
		"EFI memory map": EFI_MEMMAP_REQUEST;
		{
			memmap;
			memmap_size;
			desc_size;
			desc_version;
		}
	);
	verify!(
		(required: false)
		"EFI system table": EFI_SYSTBL_REQUEST;
		{
			address;
		}
	);
	info!(event:"limine-boot",concat!("verifying ","firmware type","..."));
	if let Some(resp) = FIRMWARE_TYPE_REQUEST.get_response()
	{
		let _tmp = resp.firmware_type();
		if _tmp == (FirmwareType::UEFI_32) || _tmp == (FirmwareType::UEFI_64)
		{
			info!(
				event: "limine-boot",
				concat!(
					"\t",
					stringify!(firmware_type),
					": {:#?}"
				), firmware_type_string(_tmp)
			);
		}
		else
		{
			bad!(
				concat!(
					"\t",
					"expected `{}` for response field `",
					stringify!(firmware_type),
					"`, but got `{:#?}` instead"
				),
				stringify!((FirmwareType::UEFI_32) || (FirmwareType::UEFI_64)),
				firmware_type_string(_tmp)
			);
		};
	}
	else
	{
		very_bad!(concat!(
			"couldn't verify ",
			"firmware type",
			" response: ",
			stringify!(FIRMWARE_TYPE_REQUEST),
			" has no response field !"
		));
	};

	verify!(
		"framebuffer": FRAMEBUFFER_REQUEST;
		{
			/* framebuffers; */
		}
	);
	verify!(
		"higher-half direct map": HHDM_REQUEST;
		{
			offset;
		}
	);
	verify!(
		"kernel addresses": KERNEL_ADDRESS_REQUEST;
		{
			virtual_base = 0xffffffff80000000_usize.as_();
			physical_base;
		}
	);
	verify!(
		"kernel cmdline": KERNEL_CMDLINE_REQUEST;
		{
			cmdline;
		}
	);
	verify!(
		"Limine memory map": MEMMAP_REQUEST;
		{
			// TODO: print them properly
			/* entries; */
		}
	);
	verify!(
		"modules": MODULES_REQUEST;
		{
			/* modules; */
		}
	);
	verify!(
		"cpu cores": MP_REQUEST;
		{
			bsp_lapic_id;
			/* flags; */
		}
	);
	verify!(
		"paging": PAGING_MODE_REQUEST;
		{
			/* mode; */
		}
	);
	verify!(
		(required: false)
		"rsdp": RSDP_REQUEST;
		{
			address;
		}
	);
	verify!(
		(required: false)
		"smbios": SMBIOS_REQUEST;
		{
			entry_32;
			entry_64;
		}
	);
	verify!(
		"stack size": STACK_SIZE_REQUEST;
		{}
	);
	true
}

mod entry
{
	use super::*;
	use crate::{
		info,
		init::{self, ctors::CtorIter},
		kernel::linker::map::zerOS_kernel_start,
		kmain,
		trace
	};

	#[unsafe(no_mangle)]
	extern "sysv64" fn zerOS_boot_setup() -> !
	{
		// All limine requests must also be referenced in a called function, otherwise
		// they may be removed by the linker.
		assert!(BASE_REVISION.is_supported());
		assert_eq!(
			&raw const zerOS_kernel_start as usize,
			0xffffffff80000000_usize
		);

		CtorIter::new().for_each(|ctor| unsafe { ctor() });

		log::set_max_level(log::LevelFilter::Warn);

		{
			let mut guard = init::cmdline::ZEROS_COMMAND_LINE.write();
			let cmdline = &mut *guard;
			*cmdline = KERNEL_CMDLINE_REQUEST
				.get_response()
				.unwrap()
				.cmdline()
				.to_str()
				.unwrap()
				.into();
		}

		let loglvl_wanted = init::cmdline::ZEROS_COMMAND_LINE.read().log_level;
		log::set_max_level(loglvl_wanted);
		info!("log level set to {loglvl_wanted}");

		assert!(verify_requests());

		if let Some(f) = MODULES_REQUEST
			.get_response()
			.unwrap()
			.modules()
			.iter()
			.find(|file| file.path().to_str().unwrap().ends_with("debug-info.zko"))
		{
			trace!("feeding debug info to kernel unwinder");
			todo!("do something with f ({})", f.path().to_string_lossy());
		}
		else
		{
			warn!("no debug info to feed to kernel unwinder !");
		}

		info!("initializing GDT...");
		init::memory::gdt::init();
		info!("GDT initialized");

		kmain()
	}
}
