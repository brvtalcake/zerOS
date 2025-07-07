pub use super::x86_common::*;

pub fn early_init()
{
	// TODO: retrieve ProcessorCapacityAndFeatureInfo (`.linear_address_bits()`)
	// TODO: `zerOS_sign_extend_addr`/`zerOS_zero_extend_addr`/
	// `zerOS_canonical_addr` functions based on the retrieved result
	// TODO: fix the subregion xored linked list so that it uses always
	// zero-extended addresses
}
