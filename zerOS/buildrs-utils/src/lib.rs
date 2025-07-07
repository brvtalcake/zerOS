#![allow(non_snake_case)]
#![feature(exit_status_error)]

use std::path::PathBuf;

pub mod aliases;
pub mod cargo;
pub mod cc;
pub mod format;
pub mod linker;

pub fn get_topdir() -> PathBuf
{
	PathBuf::from(env!("ZEROS_TOPDIR"))
		.canonicalize()
		.expect("could not resolve path to zerOS top level directory")
}

pub fn get_scripts() -> PathBuf
{
	get_topdir().join("scripts")
}
