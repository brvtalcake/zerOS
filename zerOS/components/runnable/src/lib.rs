#![allow(non_snake_case)]
#![feature(write_all_vectored)]

use std::{
	env,
	ffi::OsStr,
	fs,
	io::{IoSlice, Write},
	mem,
	os::unix::fs::PermissionsExt,
	path,
	process::{self, Command}
};

use proc_macro::TokenStream;
use proc_macro2::{
	Delimiter as Delimiter2,
	Span as Span2,
	TokenStream as TokenStream2,
	TokenTree as TokenTree2
};
use quote::quote;
use syn::{
	Attribute,
	Generics,
	Ident,
	StaticMutability,
	Token,
	Type,
	Visibility,
	ext::IdentExt,
	parse::{Parse, ParseStream, discouraged::Speculative},
	parse_macro_input,
	token::{Brace, Paren}
};
use which::which;

mod kw
{
	use syn::custom_keyword;

	custom_keyword!(body);
}

#[allow(dead_code)]
struct ItemConstRunnable
{
	attrs:       Vec<Attribute>,
	vis:         Visibility,
	const_token: Token![const],
	ident:       Ident,
	generics:    Generics,
	colon_token: Token![:],
	ty:          Box<Type>,
	eq_token:    Token![=],
	body:        kw::body,
	exclam:      Token![!],
	braces:      Brace,
	code:        TokenStream2,
	semi_token:  Token![;]
}

macro_rules! get_braced_raw {
	($buf:ident in $input:ident) => {{
		$buf = braced_raw($input)?;
		Brace(Span2::call_site())
	}};
}

macro_rules! get_parenthesized_raw {
	($buf:ident in $input:ident) => {{
		$buf = parenthesized_raw($input)?;
		Paren(Span2::call_site())
	}};
}

macro_rules! expect_delimited_group_raw {
	($input:ident, $delim:ident) => {
		$input.step(|cursor| {
			let rest = *cursor;
			if let Some((tt, next)) = rest.token_tree()
			{
				match &tt
				{
					TokenTree2::Group(delim) if matches!(delim.delimiter(), Delimiter2::$delim) =>
					{
						return Ok((delim.stream(), next));
					},
					_ =>
					{}
				}
			}
			Err(cursor.error(format!(
				"expected {}-delimited group",
				String::from(stringify!($delim)).to_lowercase()
			)))
		})
	};
}

fn braced_raw(input: ParseStream) -> syn::Result<TokenStream2>
{
	expect_delimited_group_raw!(input, Brace)
}

fn parenthesized_raw(input: ParseStream) -> syn::Result<TokenStream2>
{
	expect_delimited_group_raw!(input, Parenthesis)
}

impl Parse for ItemConstRunnable
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let code_content;
		Ok(Self {
			attrs:       input.call(Attribute::parse_outer)?,
			vis:         input.parse()?,
			const_token: input.parse()?,
			ident:       input.parse()?,
			generics:    input.parse()?,
			colon_token: input.parse()?,
			ty:          input.parse()?,
			eq_token:    input.parse()?,
			body:        input.parse()?,
			exclam:      input.parse()?,
			braces:      get_braced_raw!(code_content in input),
			code:        code_content,
			semi_token:  input.parse()?
		})
	}
}

#[allow(dead_code)]
struct ItemStaticRunnable
{
	attrs:        Vec<Attribute>,
	vis:          Visibility,
	static_token: Token![static],
	mutability:   StaticMutability,
	ident:        Ident,
	colon_token:  Token![:],
	ty:           Box<Type>,
	eq_token:     Token![=],
	body:         kw::body,
	exclam:       Token![!],
	braces:       Brace,
	code:         TokenStream2,
	semi_token:   Token![;]
}

impl Parse for ItemStaticRunnable
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let code_content;
		Ok(Self {
			attrs:        input.call(Attribute::parse_outer)?,
			vis:          input.parse()?,
			static_token: input.parse()?,
			mutability:   input.parse()?,
			ident:        input.parse()?,
			colon_token:  input.parse()?,
			ty:           input.parse()?,
			eq_token:     input.parse()?,
			body:         input.parse()?,
			exclam:       input.parse()?,
			braces:       get_braced_raw!(code_content in input),
			code:         code_content,
			semi_token:   input.parse()?
		})
	}
}

enum RunnableItem
{
	Const(ItemConstRunnable),
	Static(ItemStaticRunnable)
}

enum RunnableLang
{
	C
	{
		compiler: String,
		std:      String,
		cc_opts:  Vec<String>
	},
	Cxx
	{
		compiler: String,
		std:      String,
		cc_opts:  Vec<String>
	},
	Rust
	{
		compiler:   String,
		rustc_opts: Vec<String>
	},
	Shell
	{
		shell:      String,
		shell_opts: Vec<String>
	}
}

impl RunnableLang
{
	fn format_extension(&self) -> &'_ str
	{
		match self
		{
			Self::C { .. } => "c",
			Self::Cxx { .. } => "cc",
			Self::Rust { .. } => "rs",
			Self::Shell { shell, .. } => shell.as_str()
		}
	}

	fn set_std<'a, T>(&mut self, new_std: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_std
		{
			Some(new_value) =>
			{
				if let Self::C { ref mut std, .. } = *self
				{
					*std = new_value.into();
				}
				else if let Self::Cxx { ref mut std, .. } = *self
				{
					*std = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn set_cc_opts(&mut self, new_cc_opts: &Vec<String>) -> &mut Self
	{
		if let Self::C {
			ref mut cc_opts, ..
		} = *self
		{
			*cc_opts = new_cc_opts.iter().map(|el| el.into()).collect();
		}
		else if let Self::Cxx {
			ref mut cc_opts, ..
		} = *self
		{
			*cc_opts = new_cc_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_rustc_opts(&mut self, new_rustc_opts: &Vec<String>) -> &mut Self
	{
		if let Self::Rust {
			ref mut rustc_opts, ..
		} = *self
		{
			*rustc_opts = new_rustc_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_shell_opts(&mut self, new_shell_opts: &Vec<String>) -> &mut Self
	{
		if let Self::Shell {
			ref mut shell_opts, ..
		} = *self
		{
			*shell_opts = new_shell_opts.iter().map(|el| el.into()).collect();
		}
		self
	}

	fn set_shell<'a, T>(&mut self, new_shell: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_shell
		{
			Some(new_value) =>
			{
				if let Self::Shell { ref mut shell, .. } = *self
				{
					*shell = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn set_compiler<'a, T>(&mut self, new_compiler: Option<&'a T>) -> &mut Self
	where
		String: From<&'a T>
	{
		match new_compiler
		{
			Some(new_value) =>
			{
				if let Self::C {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				else if let Self::Cxx {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				else if let Self::Rust {
					ref mut compiler, ..
				} = *self
				{
					*compiler = new_value.into();
				}
				self
			},
			_ => self
		}
	}

	fn default_for(lang: &'_ str) -> Result<Self, String>
	{
		match lang
		{
			"C" =>
			{
				Ok(Self::C {
					compiler: "gcc".into(),
					std:      "gnu23".into(),
					cc_opts:  vec![]
				})
			},
			"C++" | "CXX" =>
			{
				Ok(Self::Cxx {
					compiler: "g++".into(),
					std:      "gnu++23".into(),
					cc_opts:  vec![]
				})
			},
			"RUST" =>
			{
				Ok(Self::Rust {
					compiler:   "rustc".into(),
					rustc_opts: vec![]
				})
			},
			"SHELL" =>
			{
				Ok(Self::Shell {
					shell:      "bash".into(),
					shell_opts: vec![]
				})
			},
			_ => Err(format!("unknown language {lang}"))
		}
	}
}

enum RunnableExpandKind
{
	Output,
	StdOut,
	StdErr,
	ExitCode
}

struct RunnableArgs
{
	lang:   RunnableLang,
	args:   Vec<String>,
	expand: RunnableExpandKind
}

impl Parse for RunnableArgs
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		// #[runnable(lang(c++), std(gnu++23), args(blah blah) )]
		let mut comp_or_interp = None;
		let mut comp_or_interp_options = vec![];
		let mut language = RunnableLang::default_for("SHELL").unwrap();
		let mut standard = None;
		let mut exe_args = vec![];
		let mut found_lang = false;
		let mut expand = None;

		loop
		{
			let ident = input.call(Ident::parse_any).unwrap();
			match ident.to_string().to_uppercase().as_str()
			{
				"EXPAND" =>
				{
					if expand.is_some()
					{
						return Err(syn::Error::new_spanned(
							ident,
							"the expand parameter must either be supplied only once or never"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					expand = Some(match buf.to_string().to_uppercase().as_str()
					{
						"OUTPUT" => RunnableExpandKind::Output,
						"STDOUT" => RunnableExpandKind::StdOut,
						"STDERR" => RunnableExpandKind::StdErr,
						"EXITCODE" | "EXIT_CODE" => RunnableExpandKind::ExitCode,
						val =>
						{
							return Err(syn::Error::new_spanned(
								ident,
								format!("unknown expand parameter value `{}`", val)
							));
						}
					});
				},
				"LANG" | "LANGUAGE" =>
				{
					if found_lang
					{
						return Err(syn::Error::new_spanned(
							ident,
							"language parameter must be specified only once"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					let default =
						RunnableLang::default_for(buf.to_string().to_uppercase().as_str())
							.map_err(|s| syn::Error::new_spanned(ident, s))?;
					found_lang = true;
					language = default;
					language
						.set_cc_opts(&comp_or_interp_options)
						.set_rustc_opts(&comp_or_interp_options)
						.set_shell_opts(&comp_or_interp_options)
						.set_std(standard.as_ref())
						.set_shell(comp_or_interp.as_ref())
						.set_compiler(comp_or_interp.as_ref());
				},
				"STD" | "STANDARD" =>
				{
					if standard.is_some()
					{
						return Err(syn::Error::new_spanned(
							ident,
							"the standard parameter must either be supplied only once or never"
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					standard = Some(buf.to_string());
					if found_lang
					{
						language.set_std(standard.as_ref());
					}
				},
				"CCOPTS"
				| "CC_OPTS"
				| "RUSTC_OPTS"
				| "RUSTCOPTS"
				| "SH_OPTS"
				| "SHOPTS"
				| "SHELL_OPTS"
				| "SHELLOPTS"
				| "COMPILER_OPTIONS"
				| "INTERPRETER_OPTIONS" =>
				{
					let buf;
					get_parenthesized_raw!(buf in input);
					comp_or_interp_options.extend(
						buf.to_string()
							.split_whitespace()
							.map(|s| s.to_string())
							.collect::<Vec<_>>()
					);
					if found_lang
					{
						language
							.set_cc_opts(&comp_or_interp_options)
							.set_rustc_opts(&comp_or_interp_options)
							.set_shell_opts(&comp_or_interp_options);
					}
				},
				"CC" | "RUSTC" | "SH" | "SHELL" | "COMPILER" | "INTERPRETER" =>
				{
					if comp_or_interp.is_some()
					{
						return Err(syn::Error::new_spanned(
							&ident,
							format!(
								"the {} parameter must either be supplied only once or never \
								 (defaulted)",
								ident.to_string()
							)
						));
					}
					let buf;
					get_parenthesized_raw!(buf in input);
					comp_or_interp = Some(buf.to_string());
					if found_lang
					{
						language
							.set_compiler(comp_or_interp.as_ref())
							.set_shell(comp_or_interp.as_ref());
					}
				},
				"ARGS" =>
				{
					let buf;
					get_parenthesized_raw!(buf in input);
					exe_args.extend(
						buf.to_string()
							.split_whitespace()
							.map(|s| s.to_string())
							.collect::<Vec<_>>()
					);
				},
				_ =>
				{
					return Err(syn::Error::new_spanned(
						&ident,
						format!("unknown parameter `{}`", ident.to_string())
					));
				}
			}
			if input.parse::<Token![,]>().is_err()
			{
				break;
			}
		}
		Ok(Self {
			lang:   language,
			args:   exe_args,
			expand: expand.unwrap_or(RunnableExpandKind::Output)
		})
	}
}

#[allow(dead_code)]
fn longest_existing_subpath(p: &path::PathBuf) -> Option<&path::Path>
{
	for sub in p.ancestors()
	{
		if sub.exists()
		{
			return Some(sub);
		}
	}
	None
}

fn make_tempfile_path(template: impl AsRef<path::Path>) -> Option<path::PathBuf>
{
	let mut buf = fs::canonicalize(env::temp_dir()).ok()?;
	let mut rand = getrandom::u64().ok()?;

	for component in template.as_ref()
	{
		let mut bytes = component.as_encoded_bytes().to_owned();
		for b in bytes.as_mut_slice()
		{
			if *b == b'X'
			{
				let ch = (rand % 16) as u8;
				let ch = if ch < 10 { b'0' + ch } else { b'a' + (ch - 10) };
				*b = ch;

				rand /= 16;
				rand = if rand > 0
				{
					rand
				}
				else
				{
					getrandom::u64().ok()?
				};
			}
		}
		buf.push(unsafe { OsStr::from_encoded_bytes_unchecked(bytes.as_slice()) });
	}

	Some(buf)
}

fn make_tempfile(template: impl AsRef<path::Path>) -> Option<(path::PathBuf, fs::File)>
{
	let path = make_tempfile_path(template)?;
	let dir = path.parent()?;
	fs::create_dir_all(dir).ok()?;
	let file = fs::File::create_new(&path).ok()?;
	Some((path, file))
}

impl RunnableItem
{
	fn get_item_decl_start(&self) -> TokenStream2
	{
		match self
		{
			Self::Const(ItemConstRunnable {
				attrs,
				vis,
				const_token,
				ident,
				generics,
				ty,
				..
			}) =>
			{
				quote! {
					#(#attrs)* #vis #const_token #ident #generics: #ty
				}
			},
			Self::Static(ItemStaticRunnable {
				attrs,
				vis,
				static_token,
				mutability,
				ident,
				ty,
				..
			}) =>
			{
				quote! {
					#(#attrs)* #vis #static_token #mutability #ident: #ty
				}
			}
		}
	}

	fn get_code(&self) -> &TokenStream2
	{
		match self
		{
			Self::Const(ItemConstRunnable { code, .. }) => code,
			Self::Static(ItemStaticRunnable { code, .. }) => code
		}
	}

	fn execute_clike_code(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		outfile_path: &path::Path,
		compiler: &String,
		std: &String,
		cc_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream2>
	{
		if let Err(ioerr) = srcfile.write_all(self.get_code().to_string().as_bytes())
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		mem::drop(srcfile);
		let output = Command::new(compiler)
			.args([
				format!("-std={std}").as_ref(),
				srcfile_path.as_os_str(),
				"-o".as_ref(),
				outfile_path.as_os_str()
			])
			.args(cc_opts)
			.output();
		if let Err(error) = output
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to execute compiler: {}", error)
			)
			.to_compile_error());
		}
		let output = output.unwrap();
		if !output.status.success()
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!(
					"compilation failed:\n{}",
					String::from_utf8_lossy(&output.stderr)
				)
			)
			.to_compile_error());
		}
		Ok(Command::new(outfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span2::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_shell(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		shell: &String,
		shell_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream2>
	{
		let env = which("env").expect("could not find `env` executable");
		let header = format!(
			"#!{} -S {shell} {}\n\n",
			env.to_string_lossy(),
			shell_opts.join(" ")
		);
		let code = self.get_code().to_string();
		let mut to_write = [
			IoSlice::new(header.as_bytes()),
			IoSlice::new(code.as_bytes())
		];
		if let Err(ioerr) = srcfile.write_all_vectored(&mut to_write)
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		let mut perms = srcfile.metadata().unwrap().permissions();
		perms.set_mode(perms.mode() | 0o100);
		srcfile.set_permissions(perms).unwrap();
		mem::drop(srcfile);
		Ok(Command::new(srcfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span2::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_rust_code(
		&self,
		mut srcfile: fs::File,
		srcfile_path: &path::Path,
		outfile_path: &path::Path,
		compiler: &String,
		rustc_opts: &Vec<String>,
		args: &Vec<String>
	) -> Result<process::Output, TokenStream2>
	{
		if let Err(ioerr) = srcfile.write_all(self.get_code().to_string().as_bytes())
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		if let Err(ioerr) = srcfile.flush().and_then(|_| srcfile.sync_all())
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to write to temporary source file: {}", ioerr)
			)
			.to_compile_error());
		}
		drop(srcfile);
		let output = Command::new(compiler)
			.args([
				srcfile_path.as_os_str(),
				"-o".as_ref(),
				outfile_path.as_os_str()
			])
			.args(rustc_opts)
			.output();
		if let Err(error) = output
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!("unable to execute compiler: {}", error)
			)
			.to_compile_error());
		}
		let output = output.unwrap();
		if !output.status.success()
		{
			return Err(syn::Error::new(
				Span2::call_site(),
				format!(
					"compilation failed:\n{}",
					String::from_utf8_lossy(&output.stderr)
				)
			)
			.to_compile_error());
		}
		Ok(Command::new(outfile_path.as_os_str())
			.args(args)
			.output()
			.map_err(|err| {
				syn::Error::new(Span2::call_site(), format!("couldn't spawn process: {err}"))
					.to_compile_error()
			})?)
	}

	fn execute_code(&self, exec_info: &RunnableArgs) -> TokenStream2
	{
		let &RunnableArgs { lang, args, expand } = &exec_info;
		let (srcfile_path, srcfile) = match make_tempfile(format!(
			"proc-macro-utils/generated/runnable-XXXXXXXX.{}",
			lang.format_extension()
		))
		{
			Some(okres) => okres,
			_ =>
			{
				return syn::Error::new(
					Span2::call_site(),
					"unable to create temporary source file !"
				)
				.to_compile_error();
			}
		};
		let outfile_path = match make_tempfile_path("proc-macro-utils/generated/runnable-XXXXXXXX")
		{
			Some(okres) => okres,
			_ =>
			{
				return syn::Error::new(
					Span2::call_site(),
					"unable to create temporary source file !"
				)
				.to_compile_error();
			}
		};
		let process_output = match lang
		{
			&RunnableLang::C {
				ref compiler,
				ref std,
				ref cc_opts
			} =>
			{
				self.execute_clike_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					std,
					cc_opts,
					args
				)
			},
			&RunnableLang::Cxx {
				ref compiler,
				ref std,
				ref cc_opts
			} =>
			{
				self.execute_clike_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					std,
					cc_opts,
					args
				)
			},
			&RunnableLang::Rust {
				ref compiler,
				ref rustc_opts
			} =>
			{
				self.execute_rust_code(
					srcfile,
					&srcfile_path,
					&outfile_path,
					compiler,
					rustc_opts,
					args
				)
			},
			&RunnableLang::Shell {
				ref shell,
				ref shell_opts
			} => self.execute_shell(srcfile, &srcfile_path, shell, shell_opts, args)
		};

		if let Err(err) = process_output.clone().and_then(|output| {
			output.status.code().ok_or(
				syn::Error::new(Span2::call_site(), "the process didn't terminate normally")
					.to_compile_error()
			)
		})
		{
			return err;
		}

		let mut process_output = process_output.unwrap();

		match expand
		{
			RunnableExpandKind::ExitCode =>
			{
				let exit_code = process_output.status.code();
				quote! { #exit_code }
			},
			RunnableExpandKind::Output =>
			{
				let mut output = process_output.stderr;
				output.append(&mut process_output.stdout);
				let output = String::from_utf8_lossy(&output);
				quote! { #output }
			},
			RunnableExpandKind::StdOut =>
			{
				let stdout = String::from_utf8_lossy(&process_output.stdout);
				quote! { #stdout }
			},
			RunnableExpandKind::StdErr =>
			{
				let stderr = String::from_utf8_lossy(&process_output.stderr);
				quote! { #stderr }
			}
		}
	}
}

impl Parse for RunnableItem
{
	fn parse(input: ParseStream) -> syn::Result<Self>
	{
		let mut errors;

		let fork = input.fork();
		match (&fork, fork.parse())
		{
			(forked, Ok(itconst)) =>
			{
				input.advance_to(forked);
				return Ok(Self::Const(itconst));
			},
			(_, Err(err)) => errors = err
		}

		input
			.parse()
			.map_err(|err| {
				errors.combine(err);
				errors.to_owned()
			})
			.map(|it| Self::Static(it))
	}
}

/// # TODO
/// - change the needed `body! { ... }` wrapping macro to `stringify!` its args
/// - then get the raw string instead of a `TokenStream2`, and see if it enables
///   us to get the input unchanged
/// - else, change the syntax to make it look like an `asm!` statement, i.e. a
///   `#[proc_macro]` instead of a `#[proc_macro_attribute]`, and with
///   `options(...)`, etc...
#[proc_macro_attribute]
pub fn runnable(input: TokenStream, annotated_item: TokenStream) -> TokenStream
{
	let item = parse_macro_input!(annotated_item as RunnableItem);
	let args = parse_macro_input!(input as RunnableArgs);

	let decl_start = item.get_item_decl_start();
	let expansion = item.execute_code(&args);

	quote! { #decl_start = #expansion; }.into()
}
