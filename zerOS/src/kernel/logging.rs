/// ! TODO: implement key-value logging: https://docs.rs/log/0.4.27/log/kv/index.html
/// ! TODO: implement per-subsystem logging using either kw logging or just
/// `target`
use core::sync::atomic::{self, AtomicBool};

use anyhow::anyhow;
use lazy_static::lazy_static;

use crate::kernel::{io::KernelOutput, sync::BasicMutex};

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! log {
    // log!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "log");
    (logger: $logger:expr, event: $event:expr, $lvl:expr, $($arg:tt)+) => ({
        ::log::log!(
            logger: $logger,
            target: $event,
            $lvl,
            $($arg)+
        )
    });

    // log!(logger: my_logger, Level::Info, "a log event")
    (logger: $logger:expr, $lvl:expr, $($arg:tt)+) => ({
        ::log::log!(
            logger: $logger,
            target: "",
            $lvl,
            $($arg)+
        )
    });

    // log!(event: "my_event", Level::Info, "a log event")
    (event: $event:expr, $lvl:expr, $($arg:tt)+) => ({
        ::log::log!(
            target: $event,
            $lvl,
            $($arg)+
        )
    });

    // log!(Level::Info, "a log event")
    ($lvl:expr, $($arg:tt)+) => ({
        ::log::log!(
            $lvl,
            $($arg)+
        )
    });
}

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! error {
    // error!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "error");
    (logger: $logger:expr, event: $event:expr, $($arg:tt)+) => ({
        ::log::error!(
            logger: $logger,
            target: $event,
            $($arg)+
        )
    });

    // error!(logger: my_logger, Level::Info, "a error event")
    (logger: $logger:expr, $($arg:tt)+) => ({
        ::log::error!(
            logger: $logger,
            target: "",
            $($arg)+
        )
    });

    // error!(event: "my_event", Level::Info, "a error event")
    (event: $event:expr, $($arg:tt)+) => ({
        ::log::error!(
            target: $event,
            $($arg)+
        )
    });

    // error!(Level::Info, "a error event")
    ($($arg:tt)+) => ({
        ::log::error!(
            target: "",
            $($arg)+
        )
    });
}

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! warn {
    // warn!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "warn");
    (logger: $logger:expr, event: $event:expr, $($arg:tt)+) => ({
        ::log::warn!(
            logger: $logger,
            target: $event,
            $($arg)+
        )
    });

    // warn!(logger: my_logger, Level::Info, "a warn event")
    (logger: $logger:expr, $($arg:tt)+) => ({
        ::log::warn!(
            logger: $logger,
            target: "",
            $($arg)+
        )
    });

    // warn!(event: "my_event", Level::Info, "a warn event")
    (event: $event:expr, $($arg:tt)+) => ({
        ::log::warn!(
            target: $event,
            $($arg)+
        )
    });

    // warn!(Level::Info, "a warn event")
    ($($arg:tt)+) => ({
        ::log::warn!(
            target: "",
            $($arg)+
        )
    });
}

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! info {
    // info!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "info");
    (logger: $logger:expr, event: $event:expr, $($arg:tt)+) => ({
        ::log::info!(
            logger: $logger,
            target: $event,
            $($arg)+
        )
    });

    // info!(logger: my_logger, Level::Info, "a info event")
    (logger: $logger:expr, $($arg:tt)+) => ({
        ::log::info!(
            logger: $logger,
            target: "",
            $($arg)+
        )
    });

    // info!(event: "my_event", Level::Info, "a info event")
    (event: $event:expr, $($arg:tt)+) => ({
        ::log::info!(
            target: $event,
            $($arg)+
        )
    });

    // info!(Level::Info, "a info event")
    ($($arg:tt)+) => ({
        ::log::info!(
            target: "",
            $($arg)+
        )
    });
}

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! debug {
    // debug!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "debug");
    (logger: $logger:expr, event: $event:expr, $($arg:tt)+) => ({
        ::log::debug!(
            logger: $logger,
            target: $event,
            $($arg)+
        )
    });

    // debug!(logger: my_logger, Level::Info, "a debug event")
    (logger: $logger:expr, $($arg:tt)+) => ({
        ::log::debug!(
            logger: $logger,
            target: "",
            $($arg)+
        )
    });

    // debug!(event: "my_event", Level::Info, "a debug event")
    (event: $event:expr, $($arg:tt)+) => ({
        ::log::debug!(
            target: $event,
            $($arg)+
        )
    });

    // debug!(Level::Info, "a debug event")
    ($($arg:tt)+) => ({
        ::log::debug!(
            target: "",
            $($arg)+
        )
    });
}

/// copied and adapted from `log` crate source code
#[macro_export]
macro_rules! trace {
    // trace!(logger: my_logger, event: "my_event", Level::Info, "a {} event", "trace");
    (logger: $logger:expr, event: $event:expr, $($arg:tt)+) => ({
        ::log::trace!(
            logger: $logger,
            target: $event,
            $($arg)+
        )
    });

    // trace!(logger: my_logger, Level::Info, "a trace event")
    (logger: $logger:expr, $($arg:tt)+) => ({
        ::log::trace!(
            logger: $logger,
            target: "",
            $($arg)+
        )
    });

    // trace!(event: "my_event", Level::Info, "a trace event")
    (event: $event:expr, $($arg:tt)+) => ({
        ::log::trace!(
            target: $event,
            $($arg)+
        )
    });

    // trace!(Level::Info, "a trace event")
    ($($arg:tt)+) => ({
        ::log::trace!(
            target: "",
            $($arg)+
        )
    });
}

pub static ZEROS_GLOBAL_LOGGER: MultiLogger = MultiLogger::new();

ctor! {
	@name(zerOS_initialize_global_logger);
	@priority(1);

	crate::arch::target::cpu::irq::disable();
	unsafe {
		log::set_logger_racy(&ZEROS_GLOBAL_LOGGER).unwrap_or_else(
			|_| {
				crate::arch::target::cpu::irq::enable();
				crate::arch::target::cpu::misc::hcf()
			}
		);
	}
	crate::arch::target::cpu::irq::enable();
}

pub const MAX_LOGGER_COUNT: usize = 30;

#[repr(usize)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum LoggingBackend
{
	Serial = 0,
	FrameBuffer,
	DebugCon
}

static mut ENABLED_LOGGING_BACKENDS: [AtomicBool; core::mem::variant_count::<LoggingBackend>()] =
	[const { AtomicBool::new(false) }; _];

pub fn set_global_backend_state(backend: LoggingBackend, enabled: bool)
{
	unsafe {
		ENABLED_LOGGING_BACKENDS[backend as usize].store(enabled, atomic::Ordering::Release);
	}
}

pub trait LoggingEventFilter = for<'a> Fn(&'a str) -> bool;

struct Logger
{
	logger:       &'static BasicMutex<dyn KernelOutput + Sync + Send>,
	event_filter: Option<&'static (dyn LoggingEventFilter + Sync + Send)>,
	backend:      LoggingBackend
}

lazy_static! {
	static ref MAX_LOG_LEVEL_STRING_REPR_WIDTH: usize = {
		unsafe {
			*[
				log::LevelFilter::Off,
				log::LevelFilter::Error,
				log::LevelFilter::Warn,
				log::LevelFilter::Info,
				log::LevelFilter::Debug,
				log::LevelFilter::Trace
			]
			.map(|lvl| lvl.as_str().len())
			.iter()
			.max()
			.unwrap_unchecked()
		}
	};
}

impl Logger
{
	fn log_event(&self, event: &str) -> bool
	{
		match self.event_filter
		{
			Some(filter) => filter(event),
			_ => true
		}
	}

	fn level_style(&self, lvl: log::Level) -> anstyle::Style
	{
		if !self.logger.lock().supports_ansi_escape_codes()
		{
			return anstyle::Style::new();
		}
		anstyle::Style::new()
			.fg_color(Some(match lvl
			{
				log::Level::Error => anstyle::AnsiColor::BrightRed.into(),
				log::Level::Warn => anstyle::AnsiColor::BrightYellow.into(),
				log::Level::Info => anstyle::AnsiColor::BrightBlue.into(),
				log::Level::Debug => anstyle::AnsiColor::BrightGreen.into(),
				log::Level::Trace => anstyle::AnsiColor::BrightWhite.into(),
				#[allow(unreachable_patterns)]
				_ => anstyle::AnsiColor::White.into()
			}))
			.effects(match lvl
			{
				log::Level::Error | log::Level::Warn =>
				{
					anstyle::Effects::BOLD | anstyle::Effects::UNDERLINE
				},
				log::Level::Info => anstyle::Effects::BOLD,
				_ => anstyle::Effects::new()
			})
	}
}

impl log::Log for Logger
{
	fn enabled(&self, metadata: &log::Metadata) -> bool
	{
		(unsafe { ENABLED_LOGGING_BACKENDS[self.backend as usize].load(atomic::Ordering::Acquire) })
			&& metadata.level().to_level_filter() <= log::max_level()
			&& self.log_event(metadata.target())
	}

	fn flush(&self)
	{
		let mut logger = self.logger.lock();
		logger
			.flush()
			.expect("error while flushing: this shouldn't happen !")
	}

	fn log(&self, record: &log::Record)
	{
		if !self.enabled(record.metadata())
		{
			return;
		}

		let lvl_style = self.level_style(record.level());

		// SAFETY: loggers are Sync + Send, and should implement writing
		// to the underlying resource in a race-free maner
		let mut logger = self.logger.lock();

		let lvl_string = record.level().as_str();
		let args = record.args();
		let _ = match (record.line(), record.file(), record.module_path())
		{
			(Some(line), Some(file), Some(modpath)) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {file}:{line} \
					 ({modpath}) - {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			(Some(line), Some(file), None) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {file}:{line} - {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			(Some(line), None, Some(modpath)) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {line} in {modpath} - \
					 {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			(None, Some(file), Some(modpath)) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {file} ({modpath}) - \
					 {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			(None, Some(file), None) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {file} - {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			(None, None, Some(modpath)) =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - in {modpath} - {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
			_ =>
			{
				logger.write_fmt(format_args!(
					"{lvl_style}[{lvl_string:<max_width$}]{lvl_style:#} - {{ unknown source \
					 location }} - {args}\n",
					max_width = MAX_LOG_LEVEL_STRING_REPR_WIDTH
				))
			},
		};
	}
}

pub struct MultiLogger
{
	loggers: BasicMutex<[Option<Logger>; MAX_LOGGER_COUNT]>
}

impl MultiLogger
{
	pub const fn new() -> Self
	{
		Self {
			loggers: BasicMutex::new(constinit_array!(
				[Option<Logger>; MAX_LOGGER_COUNT] with None
			))
		}
	}

	pub fn add_logger(
		&self,
		logger: &'static BasicMutex<dyn KernelOutput + Sync + Send>,
		event_filter: Option<&'static (dyn LoggingEventFilter + Sync + Send)>,
		backend: LoggingBackend
	) -> anyhow::Result<&Self>
	{
		let res = self
			.loggers
			.lock()
			.iter_mut()
			.find(|item| (*item).is_none())
			.map_or(
				Err(anyhow!("couldn't find any available logger slot")),
				|el| {
					el.replace(Logger {
						logger,
						event_filter,
						backend
					});
					anyhow::Ok(())
				}
			);
		res.map(|_| self)
	}
}

impl log::Log for MultiLogger
{
	fn enabled(&self, metadata: &log::Metadata) -> bool
	{
		metadata.level().to_level_filter() <= log::max_level()
	}

	fn flush(&self) {}

	fn log(&self, record: &log::Record)
	{
		if !self.enabled(record.metadata())
		{
			return;
		}

		for maybe_logger in self.loggers.lock().iter()
		{
			if let Some(logger) = maybe_logger
			{
				logger.log(record);
			}
		}
	}
}
