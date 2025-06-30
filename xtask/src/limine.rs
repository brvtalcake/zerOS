// TODO: download limine binaries manually

use std::{
	collections::HashMap,
	ffi::OsStr,
	marker::{PhantomCovariantLifetime, variance},
	pin::Pin,
	str::FromStr,
	sync::{Arc, LazyLock}
};

use anyhow::{Result, anyhow};
use camino::{Utf8Path, Utf8PathBuf};
use fmmap::tokio::{AsyncMmapFileMut, AsyncOptions};
use futures::StreamExt;
use http::header::{CONTENT_LENGTH, LOCATION};
use hyper::body::Buf;
use log::info;
use octocrab::{
	Octocrab,
	models::repos::{DiffEntry, RepoCommit}
};
use regex::Regex;
use tempfile::tempfile;
use tokio::{fs, io::AsyncWriteExt, task};
use url::Url;
use versions::Version;

use crate::{
	SupportedArch,
	requests::REQWEST_CLIENT,
	tools::{check, check_opt, mkdir}
};

fn bin_release_regex() -> &'static Regex
{
	static BIN_RELEASE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
		check!(
			Regex::new(r"v(?<version>\d+.\d+(.\d+)?)-binary")
				.expect("could not compile regular expression")
		)
	});
	&BIN_RELEASE_REGEX
}

async fn get_latest_bin_release(
	octo: &Octocrab,
	major: impl std::range::RangeBounds<usize>
) -> Result<(Version, RepoCommit)>
{
	let to_match = tokio::task::spawn_blocking(bin_release_regex);
	let limine_repo = octo.repos("limine-bootloader", "limine");
	let limine_commits = octo.commits("limine-bootloader", "limine");
	let tags = limine_repo.list_tags().send().await?;
	let mut latest = None;
	let to_match = to_match.await?;
	for (vers, commit_sha) in tags.into_iter().filter_map(|ref t| {
		to_match
			.captures(&t.name)
			.and_then(|ref captures| {
				captures
					.name("version")
					.map(|ref captured| {
						check!(
							Version::from_str(captured.as_str())
								.expect("could not coerce limine version")
						)
					})
					.zip(Some(t.commit.sha.clone()))
			})
			.filter(|(v, _)| v.nth(0).is_some_and(|maj| major.contains(&(maj as usize))))
	})
	{
		let commit = limine_commits.get(commit_sha).await?;
		if let Some((ref mut curr_version, ref mut curr_commit)) = latest
		{
			if *curr_version < vers
			{
				*curr_version = vers;
				*curr_commit = commit;
			}
		}
		else
		{
			latest = Some((vers, commit));
		}
	}
	Ok(latest.ok_or(anyhow!(
		"could not find limine binary release matching the provided constraints"
	))?)
}

#[remain::sorted]
enum DownloadUrl
{
	Content(Url),
	Raw(Url)
}

struct EntryMetadata
{
	url:              DownloadUrl,
	approximate_size: usize
}

async fn entry_meta(entry: DiffEntry) -> EntryMetadata
{
	if let Some(raw) = &entry.raw_url
	{
		let mut url = Url::from_str(&raw).expect(format!("invalid url: {raw}").as_str());
		let mut resp = check!(
			REQWEST_CLIENT
				.head(url.clone())
				.send()
				.await
				.expect(format!("could not reach url: {url}").as_str())
		);
		let mut status = resp.status();

		while status.as_u16() >= 300 && status.as_u16() < 400
		{
			let tmp_url = check!(
				check_opt!(
					resp.headers().get(LOCATION).expect(
						format!(
							"got status code {} but could not find `location` header in response",
							resp.status()
						)
						.as_str()
					)
				)
				.to_str()
				.expect("the `location` response header contains some invalid characters")
			);
			url = check!(Url::from_str(tmp_url).expect(format!("invalid url: {tmp_url}").as_str()));
			resp = check!(
				REQWEST_CLIENT
					.head(url.clone())
					.send()
					.await
					.expect(format!("could not reach url: {url}").as_str())
			);
			status = resp.status();
		}

		let size = check_opt!(
			check!(
				REQWEST_CLIENT
					.head(url.clone())
					.send()
					.await
					.expect(format!("could not reach url: {url}").as_str())
					.headers()
					.get(CONTENT_LENGTH)
					.map(|hdrval| {
						check!(
							hdrval
								.to_str()
								.expect(
									"the `content-length` response header contains some invalid \
									 characters"
								)
								.parse()
								.expect("could not parse the `content-length` response header")
						)
					})
			)
			.expect("could not get the `content-length` response header")
		);
		EntryMetadata {
			url:              DownloadUrl::Raw(url),
			approximate_size: size
		}
	}
	else
	{
		todo!()
	}
}

async fn write_content(
	entry: EntryMetadata,
	args: (Option<&'static str>, Utf8PathBuf)
) -> (Option<&'static str>, Utf8PathBuf)
{
	let (flag, path) = args;
	mkdir(
		true,
		false,
		&check_opt!(path.parent().expect("path has no parents"))
	)
	.await;
	match entry.url
	{
		DownloadUrl::Raw(url) =>
		{
			let (istream, ostream) = tokio::join!(
				tokio::spawn((async move |u| {
					check!(
						REQWEST_CLIENT
							.get(u)
							.send()
							.await
							.expect("could not download content")
							.bytes_stream()
					)
				})(url.clone())),
				tokio::spawn((async move |p| {
					fs::OpenOptions::new()
						.append(false)
						.create(true)
						.write(true)
						.read(false)
						.open(&p)
						.await
						.expect("could not open or create file")
				})(path.clone()))
			);
			let (mut istream, mut ostream) = (
				check!(istream.expect("can not create input stream")),
				check!(ostream.expect("can not create output stream"))
			);
			// TODO: maybe we could gain performance by doing reading and writing in two
			// separate tasks/threads ?
			while let Some(item) = istream.next().await
			{
				let mut bytes = check!(item.expect("could not read server body response"));
				check!(
					ostream
						.write_all_buf(&mut bytes)
						.await
						.expect("could not write to file")
				);
				debug_assert!(!bytes.has_remaining());
			}

			check!(
				ostream
					.sync_all()
					.await
					.expect("could not flush and the syncronize the file with filesystem")
			);
		},
		DownloadUrl::Content(url) =>
		{
			todo!()
		}
	}
	(flag, path)
}

fn uefi_suffix(arch: SupportedArch) -> &'static str
{
	match arch
	{
		SupportedArch::Amd64 => "X64",
		SupportedArch::X86 => "IA32",
		SupportedArch::LoongArch64 => "LOONGARCH64",
		SupportedArch::AArch64 => "AA64",
		SupportedArch::Riscv64 => "RISCV64",
		_ => unreachable!()
	}
}

pub(crate) async fn download(
	major: impl std::range::RangeBounds<usize>,
	arch: SupportedArch,
	root: &Utf8Path
) -> Vec<task::JoinHandle<(Option<&'static str>, Utf8PathBuf)>>
{
	let (vers, commit) = check!(
		get_latest_bin_release(&octocrab::instance(), major)
			.await
			.expect("could not retrieve latest limine binary release")
	);
	info!("found suitable limine version: {vers}");

	let is_needed = match arch
	{
		SupportedArch::Amd64 | SupportedArch::X86 =>
		{
			|s: &String| {
				if *s == "BOOT".to_owned() + uefi_suffix(arch) + ".EFI"
				{
					Some((None, root.join("EFI").join("BOOT").join(s)))
				}
				else if *s == "limine-bios-cd.bin"
				{
					Some((
						Some("-b"),
						root.join("boot").join("limine").join("limine-bios-cd.bin")
					))
				}
				else if *s == "limine-uefi-cd.bin"
				{
					Some((
						Some("--efi-boot"),
						root.join("boot").join("limine").join("limine-uefi-cd.bin")
					))
				}
				else
				{
					None
				}
			}
		},
		_ => todo!()
	};

	let mut spawned = vec![];
	let files = check_opt!(
		commit
			.files
			.expect("could not find any file in remote limine")
	);
	for (entry, dest) in files
		.iter()
		.cloned()
		.filter_map(|entry| Some(entry.clone()).zip(is_needed(&entry.filename)))
	{
		spawned.push(tokio::task::spawn(async move {
			write_content(entry_meta(entry).await, dest).await
		}));
	}

	spawned
}
