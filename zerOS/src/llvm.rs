unsafe extern "C" {
	#[link_name = "llvm.returnaddress"]
	pub unsafe fn return_address(a: i32) -> *const u8;

    #[link_name = "llvm.frameaddress"]
	pub unsafe fn frame_address(a: i32) -> *const u8;
}
