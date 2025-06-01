use std::{env, fs};

use anyhow::{Result, anyhow};
use target_spec_json::*;

pub fn main() -> Result<()>
{
	let args = env::args().collect::<Vec<_>>();
	let content = fs::read_to_string(args[1].clone())
		.map_err(|err| anyhow!("couldn't read {}! reason: {}", args[1], err))?;
	let spec: TargetSpec = serde_json::from_str(&content)?;
	println!("{}", serde_json::to_string_pretty(&spec)?);
	Ok(())
}
