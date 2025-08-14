use crate::clock;
use crate::rom::uart::*;
use crate::uart0;

static mut USART_DRIVER_MEMORY_BUFFER: [u8; 40] = [0u8; 40];

pub fn init_debug_logging(sys: &lpc11u6x_pac::SYSCON, uart0: lpc11u6x_pac::USART0) -> Dbg {
    let uart_dbg = Dbg::new(sys, uart0);
    uart_dbg.newline();
    uart_dbg.reset_color();

    uart_dbg
}

pub struct Dbg {
    driver: USART0Driver,
    ram_pointer: usize,
    handle: usize,
    periph: lpc11u6x_pac::USART0,
}

impl Dbg {
    pub fn new(sys: &lpc11u6x_pac::SYSCON, uart0: lpc11u6x_pac::USART0) -> Self {
        uart0::init(sys, &uart0);
        let mut cfg = UART_CONFIG {
            sys_clk_in_hz: clock::get_sys_clk(sys),
            baudrate_in_hz: 115200,
            config: 1,
            error_en: 0,
            sync_mod: 0,
        };
        let driver = unsafe { USART0Driver::new() };
        let ram_pointer: *mut u8 =
            core::ptr::addr_of_mut!(USART_DRIVER_MEMORY_BUFFER) as *mut u8;
        let handle = driver.setup(ram_pointer);
        driver.init(handle, &mut cfg);
        Dbg {
            driver,
            ram_pointer: ram_pointer as usize,
            handle: handle as usize,
            periph: uart0,
        }
    }

    pub fn reinit(&self, sys: &lpc11u6x_pac::SYSCON) {
        let mut cfg = UART_CONFIG {
            sys_clk_in_hz: clock::get_sys_clk(sys),
            baudrate_in_hz: 115200,
            config: 1,
            error_en: 0,
            sync_mod: 0,
        };
        unsafe { USART_DRIVER_MEMORY_BUFFER.fill(0) };
        let handle = self.driver.setup(self.ram_pointer as *mut u8);
        self.driver.init(handle, &mut cfg);
        self.newline();
    }

    pub fn print_raw_byte(&self, byte: u8) {
        self.driver.put_char(self.handle as *mut _, byte);
    }

    pub fn print_hex_byte(&self, byte: u8) {
        let first = byte / 16;
        let second = byte % 16;
        if first < 10 {
            self.driver.put_char(self.handle as *mut _, first + 48);
        } else {
            self.driver.put_char(self.handle as *mut _, first + 55);
        }
        if second < 10 {
            self.driver.put_char(self.handle as *mut _, second + 48);
        } else {
            self.driver.put_char(self.handle as *mut _, second + 55);
        }
    }

    pub fn print(&self, text: *const u8, len: u32) {
        let mut parm = UART_PARAM {
            buffer: text,
            dma: 0,
            dma_num: 0,
            driver_mode: 0,
            size: len,
            transfer_mode: 0,
            callback_func_pt: None,
        };
        self.driver.put_line(self.handle as *mut _, &mut parm);
    }
    pub fn println(&self, text: *const u8, len: u32) {
        let mut parm = UART_PARAM {
            buffer: text,
            dma: 0,
            dma_num: 0,
            driver_mode: 0,
            size: len,
            transfer_mode: 0,
            callback_func_pt: None,
        };
        self.driver.put_line(self.handle as *mut _, &mut parm);
        self.newline();
    }

    pub fn newline(&self) {
        self.driver.put_char(self.handle as *mut _, 10);
        self.driver.put_char(self.handle as *mut _, 13);
    }

    pub fn reset_color(&self) {
        static COLOR_CODE: &str = "\u{001b}[0m";
        let mut parm = UART_PARAM {
            buffer: COLOR_CODE.as_ptr(),
            dma: 0,
            dma_num: 0,
            driver_mode: 0,
            size: COLOR_CODE.len() as u32,
            transfer_mode: 0,
            callback_func_pt: None,
        };
        self.driver.put_line(self.handle as *mut _, &mut parm);
    }

    pub fn debug(&self, msg: &str) {
        self.println(msg.as_ptr(), msg.len() as u32);
    }

    pub fn debug_number(&self, mut num: u32) {
        let mut buffer = [0_u8; 32];
        let mut index = 31;
        while num >= 10 {
            buffer[index] = (num % 10) as u8 + 48;
            num /= 10;
            index -= 1;
        }
        buffer[index] = num as u8 + 48;
        self.println(buffer[index..].as_ptr(), buffer[index..].len() as u32);
    }

    pub fn debug_eeprom(&self, sys: &lpc11u6x_pac::SYSCON) {
        self.println("EEPROM content:".as_ptr(), 15);

        for i in 0..16 {
            let mut buffer = [0_u8; 256];
            crate::rom::eeprom::eeprom_read(0xFF * i, &mut buffer, sys);

            for (i, byte) in buffer.iter().enumerate() {
                if i % 16 == 0 {
                    self.newline();
                }
                self.print_hex_byte(*byte);
                self.print_raw_byte(32);
            }
        }
        self.newline();
    }
}
