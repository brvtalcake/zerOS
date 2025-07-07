use std::{
	path::Path,
	process::{Command, ExitStatusError}
};

pub fn format_rust_file<C: AsRef<Path>, F: AsRef<Path>>(
	config_file: C,
	source_file: F
) -> Result<(), ExitStatusError>
{
	let args = [
		"--config-path",
		config_file.as_ref().to_str().expect("invalid string"),
		source_file.as_ref().to_str().expect("invalid string")
	];
	Command::new("rustfmt")
		.args(args)
		.spawn()
		.expect("could not spawn rustfmt")
		.wait()
		.expect("could not wait rustfmt")
		.exit_ok()
}
