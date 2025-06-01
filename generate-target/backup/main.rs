use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Endian
{
	Little,
	Big
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CodeModel
{
	Tiny,
	Small,
	Kernel,
	Medium,
	Large
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RelocModel
{
	Static,
	Pic,
	Pie,
	DynamicNoPic,
	Ropi,
	Rwpi,
	RopiRwpi
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FramePointer
{
	Always,
	NonLeaf,
	MayOmit
}

/// TODO: what is `Emulated` ???
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum TlsModel
{
	GeneralDynamic,
	LocalDynamic,
	InitialExec,
	LocalExec,
	Emulated
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum BinaryFormat
{
	Coff,
	Elf,
	MachO,
	Wasm,
	Xcoff
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RelroLevel
{
	Full,
	Partial,
	Off,
	None
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PanicStrategy
{
	Unwind,
	Abort
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SymbolVisibility
{
	Hidden,
	Protected,
	Interposable
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum MergeFunctions
{
	Disabled,
	Trampolines,
	Aliases
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RustcAbi
{
	X86Sse2,
	X86Softfloat
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum DebuginfoKind
{
	Dwarf,
	DwarfDsym,
	Pdb
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum SplitDebuginfo
{
	Off,
	Packed,
	Unpacked
}

bitflags::bitflags! {
	#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
	#[serde(rename_all = "kebab-case")]
	struct SanitizerSet: u16 {
		const ADDRESS			= 1 <<  0;
		const LEAK				= 1 <<  1;
		const MEMORY			= 1 <<  2;
		const THREAD			= 1 <<  3;
		const HWADDRESS			= 1 <<  4;
		const CFI				= 1 <<  5;
		const MEMTAG			= 1 <<  6;
		const SHADOW_CALL_STACK	= 1 <<  7;
		const KCFI				= 1 <<  8;
		const KERNEL_ADDRESS	= 1 <<  9;
		const SAFESTACK			= 1 << 10;
		const DATAFLOW			= 1 << 11;
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum FloatAbi
{
	Soft,
	Hard
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Conv
{
	C,
	Rust,
	Cold,
	PreserveMost,
	PreserveAll,
	ArmAapcs,
	CCmseNonSecureCall,
	CCmseNonSecureEntry,
	Msp430Intr,
	GpuKernel,
	X86Fastcall,
	X86Intr,
	X86Stdcall,
	X86ThisCall,
	X86VectorCall,
	X86_64SysV,
	X86_64Win64,
	AvrInterrupt,
	AvrNonBlockingInterrupt,
	RiscvInterrupt
	{
		kind: RiscvInterruptKind
	}
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RiscvInterruptKind
{
	Machine,
	Supervisor
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TargetSpec
{
	llvm_target:       String,
	pointer_width:     u32,
	arch:              String,
	data_layout:       String,
	endian:            Endian,
	c_int_width:       u32,
	os:                String,
	env:               String,
	abi:               String,
	vendor:            String,
	linker:            Option<String>,
	linker_flavor:     Option<String>,
	// linker_flavor_json: LinkerFlavorCli,
	// lld_flavor_json: LldFlavor,
	// linker_is_gnu_json: bool,
	// pre_link_objects: CrtObjects,
	// post_link_objects: CrtObjects,
	// pre_link_objects_self_contained: CrtObjects,
	// post_link_objects_self_contained: CrtObjects,
	// link_self_contained: LinkSelfContainedDefault,
	// pre_link_args: LinkArgs,
	// pre_link_args_json: LinkArgsCli,
	// late_link_args: LinkArgs,
	// late_link_args_json: LinkArgsCli,
	// late_link_args_dynamic: LinkArgs,
	// late_link_args_dynamic_json: LinkArgsCli,
	// late_link_args_static: LinkArgs,
	// late_link_args_static_json: LinkArgsCli,
	// post_link_args: LinkArgs,
	// post_link_args_json: LinkArgsCli,
	// link_script: Option<String>,
	link_env:          BTreeMap<String, String>,
	link_env_remove:   Vec<String>,
	asm_args:          Vec<String>,
	cpu:               String,
	need_explicit_cpu: bool,

	/// TODO
	/// Note that these are LLVM feature names, not Rust feature names!
	/// Generally it is a bad idea to use negative target features because they
	/// often interact very poorly with how -Ctarget-cpu works. Instead, try to
	/// use a lower “base CPU” and enable the features you want to use.
	features: String,
	direct_access_external_data: Option<bool>,
	dynamic_linking: bool,
	dll_tls_export: bool,
	only_cdylib: bool,
	executables: bool,
	relocation_model: RelocModel,
	code_model: Option<CodeModel>,
	tls_model: TlsModel,
	disable_redzone: bool,
	frame_pointer: FramePointer,
	function_sections: bool,
	dll_prefix: String,
	dll_suffix: String,
	exe_suffix: String,
	staticlib_prefix: String,
	staticlib_suffix: String,
	families: Vec<String>,
	abi_return_struct_as_int: bool,
	is_like_aix: bool,
	is_like_darwin: bool,
	is_like_solaris: bool,
	is_like_windows: bool,
	is_like_msvc: bool,
	is_like_wasm: bool,
	is_like_android: bool,
	binary_format: BinaryFormat,
	default_dwarf_version: u32,
	allows_weak_linkage: bool,
	has_rpath: bool,
	no_default_libraries: bool,
	position_independent_executables: bool,
	static_position_independent_executables: bool,
	plt_by_default: bool,
	relro_level: RelroLevel,
	archive_format: String,
	allow_asm: bool,
	main_needs_argc_argv: bool,
	has_thread_local: bool,
	obj_is_bitcode: bool,
	bitcode_llvm_cmdline: String,
	min_atomic_width: Option<u64>,
	max_atomic_width: Option<u64>,
	atomic_cas: bool,
	panic_strategy: PanicStrategy,
	crt_static_allows_dylibs: bool,
	crt_static_default: bool,
	crt_static_respected: bool,
	// stack_probes: StackProbeType,
	min_global_align: Option<u64>,
	default_codegen_units: Option<u64>,
	default_codegen_backend: Option<String>,
	trap_unreachable: bool,
	requires_lto: bool,
	singlethread: bool,
	no_builtins: bool,
	default_visibility: Option<SymbolVisibility>,
	emit_debug_gdb_scripts: bool,
	requires_uwtable: bool,
	default_uwtable: bool,
	simd_types_indirect: bool,
	limit_rdylib_exports: bool,
	override_export_symbols: Option<Vec<String>>,
	merge_functions: MergeFunctions,
	mcount: String,
	llvm_mcount_intrinsic: Option<String>,
	llvm_abiname: String,
	llvm_floatabi: Option<FloatAbi>,
	rustc_abi: Option<RustcAbi>,
	relax_elf_relocations: bool,
	llvm_args: Vec<String>,
	use_ctors_section: bool,
	eh_frame_header: bool,
	has_thumb_interworking: bool,
	debuginfo_kind: DebuginfoKind,
	split_debuginfo: SplitDebuginfo,
	supported_split_debuginfo: Vec<SplitDebuginfo>,
	supported_sanitizers: SanitizerSet,
	c_enum_min_bits: Option<u64>,
	generate_arange_section: bool,
	supports_stack_protector: bool,
	entry_name: String,
	entry_abi: Conv,
	supports_xray: bool
	// small_data_threshold_support: SmallDataThresholdSupport
}

fn gen_x86_64() -> TargetSpec
{
	/*
{
    "llvm-target": "x86_64-unknown-none",
    "target-endian": "little",
    "target-pointer-width": "64",
    "target-c-int-width": "32",
    "data-layout": "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
    "arch": "x86_64",
    "os": "none",
    "env": "",
    "vendor": "unknown",
    "linker": "rust-lld",
    "linker-flavor": "gnu-lld",
    "features": "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2,+soft-float",
    "dynamic-linking": false,
    "executables": true,
    "relocation-model": "static",
    "code-model": "kernel",
    "disable-redzone": true,
    "frame-pointer": "always",
    "exe-suffix": "",
    "has-rpath": false,
    "no-default-libraries": true,
    "position-independent-executables": false,
    "rustc-abi": "x86-softfloat"
}

	 */
	TargetSpec {
		llvm_target: "x86_64-unknown-none".into(),
		pointer_width: 64,
		arch: "x86_64".into(),
		data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128".into(),
		endian: Endian::Little,
		c_int_width: 32,
		os: "none".into(),
		env: "".into(),
		abi: (),
		vendor: (),
		linker: (),
		linker_flavor: (),
		link_env: (),
		link_env_remove: (),
		asm_args: (),
		cpu: (),
		need_explicit_cpu: (),
		features: (),
		direct_access_external_data: (),
		dynamic_linking: (),
		dll_tls_export: (),
		only_cdylib: (),
		executables: (),
		relocation_model: (),
		code_model: (),
		tls_model: (),
		disable_redzone: (),
		frame_pointer: (),
		function_sections: (),
		dll_prefix: (),
		dll_suffix: (),
		exe_suffix: (),
		staticlib_prefix: (),
		staticlib_suffix: (),
		families: (),
		abi_return_struct_as_int: (),
		is_like_aix: (),
		is_like_darwin: (),
		is_like_solaris: (),
		is_like_windows: (),
		is_like_msvc: (),
		is_like_wasm: (),
		is_like_android: (),
		binary_format: (),
		default_dwarf_version: (),
		allows_weak_linkage: (),
		has_rpath: (),
		no_default_libraries: (),
		position_independent_executables: (),
		static_position_independent_executables: (),
		plt_by_default: (),
		relro_level: (),
		archive_format: (),
		allow_asm: (),
		main_needs_argc_argv: (),
		has_thread_local: (),
		obj_is_bitcode: (),
		bitcode_llvm_cmdline: (),
		min_atomic_width: (),
		max_atomic_width: (),
		atomic_cas: (),
		panic_strategy: (),
		crt_static_allows_dylibs: (),
		crt_static_default: (),
		crt_static_respected: (),
		min_global_align: (),
		default_codegen_units: (),
		default_codegen_backend: (),
		trap_unreachable: (),
		requires_lto: (),
		singlethread: (),
		no_builtins: (),
		default_visibility: (),
		emit_debug_gdb_scripts: (),
		requires_uwtable: (),
		default_uwtable: (),
		simd_types_indirect: (),
		limit_rdylib_exports: (),
		override_export_symbols: (),
		merge_functions: (),
		mcount: (),
		llvm_mcount_intrinsic: (),
		llvm_abiname: (),
		llvm_floatabi: (),
		rustc_abi: (),
		relax_elf_relocations: (),
		llvm_args: (),
		use_ctors_section: (),
		eh_frame_header: (),
		has_thumb_interworking: (),
		debuginfo_kind: (),
		split_debuginfo: (),
		supported_split_debuginfo: (),
		supported_sanitizers: (),
		c_enum_min_bits: (),
		generate_arange_section: (),
		supports_stack_protector: (),
		entry_name: (),
		entry_abi: (),
		supports_xray: ()
	}
}

fn main()
{
	println!("Hello, world!");
}
