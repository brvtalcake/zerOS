#![feature(unqualified_local_imports)]
#![feature(non_exhaustive_omitted_patterns_lint)]
#![feature(must_not_suspend)]
#![feature(multiple_supertrait_upcastable)]
#![feature(strict_provenance_lints)]
// FORBID
#![forbid(missing_unsafe_on_extern)]
#![forbid(unused_import_braces)]
#![forbid(renamed_and_removed_lints)]
// DENY
// allow by default
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![deny(lossy_provenance_casts)]
#![deny(fuzzy_provenance_casts)]
#![deny(elided_lifetimes_in_paths)]
#![deny(impl_trait_overcaptures)]
#![deny(impl_trait_redundant_captures)]
#![deny(let_underscore_drop)]
#![deny(linker_messages)]
#![deny(meta_variable_misuse)]
#![deny(multiple_supertrait_upcastable)]
#![deny(must_not_suspend)]
#![deny(non_exhaustive_omitted_patterns)]
#![deny(redundant_lifetimes)]
#![deny(single_use_lifetimes)]
#![deny(trivial_numeric_casts)]
#![deny(unit_bindings)]
#![deny(unqualified_local_imports)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_extern_crates)]
#![deny(unused_lifetimes)]
// warning
#![deny(warnings)]
// WARNING
// allow by default
#![warn(explicit_outlives_requirements)] // can cause false positives
#![warn(if_let_rescope)] // not that important
#![warn(redundant_imports)]
#![warn(unnameable_types)]
#![warn(unreachable_pub)]
#![warn(unsafe_attr_outside_unsafe)]
#![warn(unused_crate_dependencies)]
#![warn(unused_macro_rules)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
// warn by default
#![warn(incomplete_features)]
#![warn(non_camel_case_types)]
#![warn(non_snake_case)]
#![warn(non_upper_case_globals)]
// ALLOW
// warn by default
#![allow(refining_impl_trait)]
#![allow(uncommon_codepoints)]
