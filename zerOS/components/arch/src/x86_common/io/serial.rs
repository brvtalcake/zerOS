use core::{
	mem::variant_count,
	sync::atomic::{self, AtomicBool}
};

use lazy_static::lazy_static;
use zerOS_proc_macro_utils::constinit_array;

use crate::target::cpu::{
	io::{inb, outb},
	misc::hcf
};

static mut SERIAL_INIT_STATES: [AtomicBool; variant_count::<SerialPortId>() as usize] = constinit_array!([AtomicBool; variant_count::<SerialPortId>() as usize] with AtomicBool::new(false));

#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SerialPortId
{
	COM1 = 0x3f8_u16,
	COM2 = 0x2f8_u16,
	COM3 = 0x3e8_u16,
	COM4 = 0x2e8_u16,
	COM5 = 0x5f8_u16,
	COM6 = 0x4f8_u16,
	COM7 = 0x5e8_u16,
	COM8 = 0x4e8_u16
}

impl SerialPortId
{
	/// QEMU monitor
	pub const DEBUG: Self = Self::COM1;

	pub fn unique_index(&self) -> Option<usize>
	{
		match *self
		{
			Self::COM1 => Some(0),
			Self::COM2 => Some(1),
			Self::COM3 => Some(2),
			Self::COM4 => Some(3),
			Self::COM5 => Some(4),
			Self::COM6 => Some(5),
			Self::COM7 => Some(6),
			Self::COM8 => Some(7)
		}
	}
}

pub struct SerialPort
{
	id: SerialPortId
}

impl SerialPort
{
	fn is_faulty(&self) -> bool
	{
		// Set in loopback mode, test the serial chip
		outb((self.id as u16) + 4, 0x1e);

		// perform test
		#[allow(clippy::identity_op)]
		outb((self.id as u16) + 0, 0xae);
		#[allow(clippy::identity_op)]
		let ret = inb((self.id as u16) + 0) != 0xae;

		// Set back to normal operation mode
		outb((self.id as u16) + 4, 0x0f);

		ret
	}

	fn setup(&self) -> bool
	{
		let state: Result<bool, bool>;
		if unsafe {
			state = SERIAL_INIT_STATES[self.id.unique_index().unwrap()].compare_exchange(
				false,
				true,
				atomic::Ordering::AcqRel,
				atomic::Ordering::Acquire
			);
			state.clone()
		}
		.is_err()
		{
			// already initialized the COM port
			// (at least we should have)
			let ret = state.err().unwrap();
			assert_eq!(ret, true);
			return ret;
		}

		// from OSDev
		outb((self.id as u16) + 1, 0x00); // Disable all interrupts
		outb((self.id as u16) + 3, 0x80); // Enable DLAB (set baud rate divisor)
		#[allow(clippy::identity_op)]
		outb((self.id as u16) + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
		outb((self.id as u16) + 1, 0x00); //                  (hi byte)
		outb((self.id as u16) + 3, 0x03); // 8 bits, no parity, one stop bit
		outb((self.id as u16) + 2, 0xc7); // Enable FIFO, clear them, with 14-byte threshold
		outb((self.id as u16) + 4, 0x0b); // IRQs enabled, RTS/DSR set

		!self.is_faulty()
	}

	pub fn new(id: SerialPortId) -> Option<Self>
	{
		let this = Self { id };
		this.setup().then_some(this)
	}

	pub fn is_transmit_empty(&self) -> bool
	{
		inb((self.id as u16) + 5) & 0x20 != 0
	}

	pub fn serial_received(&self) -> bool
	{
		inb((self.id as u16) + 5) & 1 != 0
	}
}

impl SerialOutput for SerialPort
{
	fn supports_ansi_escape_codes(&self) -> bool
	{
		hypervisor::under_qemu().unwrap_or_else(|_| hcf()) && self.id == SerialPortId::DEBUG
	}

	fn serial_write_byte(&self, byte: u8)
	{
		while !self.is_transmit_empty()
		{
			core::hint::spin_loop();
		}
		outb(self.id as u16, byte)
	}
}

impl SerialInput for SerialPort
{
	fn serial_read_byte(&self) -> u8
	{
		while !self.serial_received()
		{
			core::hint::spin_loop();
		}

		inb(self.id as u16)
	}
}

lazy_static! {
	static ref ZEROS_COM1_SERIAL_LOGGER: BasicMutex<SerialKernelOutput<SerialPort>> =
		BasicMutex::new(SerialKernelOutput::new(
			SerialPort::new(SerialPortId::COM1).unwrap()
		));
}

ctor! {
	@name(zerOS_init_serial_loggers);
	@priority(100);
	if hypervisor::under_qemu().unwrap_or_else(|_| hcf())
	{
		ZEROS_GLOBAL_LOGGER.add_logger(
			&*ZEROS_COM1_SERIAL_LOGGER,
			None,
			logging::LoggingBackend::Serial
		).unwrap();
	}
	logging::set_global_backend_state(logging::LoggingBackend::Serial, true);
}
