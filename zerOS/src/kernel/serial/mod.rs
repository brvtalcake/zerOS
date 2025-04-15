use bitflags::bitflags;

bitflags! {
    pub struct SerialPortId : u16
    {
        const COM1 = 0x3f8_u16;
        const COM2 = 0x2f8_u16;
        const COM3 = 0x3e8_u16;
        const COM4 = 0x2e8_u16;
        const DEBUG = SerialPortId::COM1.bits();
    }
}

pub struct SerialPort
{
    id: SerialPortId,
}

impl SerialPort
{
    fn is_faulty(&self) -> bool
    {
        use crate::kernel::cpu::io::{inb, outb};

        // Set in loopback mode, test the serial chip
        outb(self.id.bits() + 4, 0x1E);

        // perform test
        outb(self.id.bits() + 0, 0xae);
        let ret = inb(self.id.bits() + 0) != 0xae;

        // Set back to normal operation mode
        outb(self.id.bits() + 4, 0x0F);

        ret
    }

    pub fn early_dbg_port() -> Option<Self>
    {
        let ret = Self {
            id: SerialPortId::DEBUG,
        };
        if ret.early_setup() { Some(ret) } else { None }
    }

    pub fn early_setup(&self) -> bool
    {
        use crate::kernel::cpu::io::outb;

        // from OSDev
        outb(self.id.bits() + 1, 0x00); // Disable all interrupts
        outb(self.id.bits() + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(self.id.bits() + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(self.id.bits() + 1, 0x00); //                  (hi byte)
        outb(self.id.bits() + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(self.id.bits() + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(self.id.bits() + 4, 0x0B); // IRQs enabled, RTS/DSR set

        !self.is_faulty()
    }
}
