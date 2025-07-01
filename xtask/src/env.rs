use std::{
	collections::HashMap,
	ffi::{OsStr, OsString},
	sync::{LazyLock, RwLock}
};

use itertools::Itertools;

use crate::tools::check;

static ENV: RwLock<LazyLock<HashMap<OsString, OsString>>> = RwLock::new(LazyLock::new(|| {
	let mut new = HashMap::new();
	for (key, value) in std::env::vars_os()
	{
		new.insert(key, value);
	}
	new
}));

pub(crate) fn var_os(key: impl AsRef<OsStr>) -> Option<OsString>
{
	check!(ENV.read().expect("lock is poisoned"))
		.get(key.as_ref())
		.cloned()
}

pub(crate) fn var(key: impl AsRef<OsStr>) -> Option<String>
{
	var_os(key).map(|s| {
		check!(
			s.into_string()
				.map_err(|err| err.display().to_string())
				.expect("could not convert from OsString to String")
		)
	})
}

pub(crate) fn vars_os() -> HashMap<OsString, OsString>
{
	check!(ENV.read().expect("lock is poisoned")).clone()
}

pub(crate) fn vars() -> HashMap<String, String>
{
	check!(ENV.read().expect("lock is poisoned"))
		.iter()
		.map(|(k, v)| {
			(
				check!(
					k.clone()
						.into_string()
						.map_err(|err| err.display().to_string())
						.expect("could not convert from OsString to String")
				),
				check!(
					v.clone()
						.into_string()
						.map_err(|err| err.display().to_string())
						.expect("could not convert from OsString to String")
				)
			)
		})
		.collect()
}

pub(crate) fn set_var(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Option<OsString>
{
	let mut guard = check!(ENV.write().expect("lock is poisoned"));
	unsafe {
		std::env::set_var(&key, &value);
	}
	guard.insert(key.as_ref().to_owned(), value.as_ref().to_owned())
}
