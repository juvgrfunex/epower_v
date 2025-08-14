use super::{CVoid, ErrorCode, ROM_API_BASE_ADR, UASRT0_OFFSET};
pub type UartHandle = CVoid;
pub type UartCallback = Option<unsafe extern "C" fn(err_code: u32, n: u32)>;
pub struct USART0Driver {
    base_addr: u32,
    uart_get_mem_size: extern "C" fn() -> u32,
    uart_setup: extern "C" fn(base_addr: u32, ram: *mut u8) -> *mut UartHandle,
    uart_init: extern "C" fn(handle: *mut UartHandle, set: *mut UART_CONFIG) -> u32,
    uart_put_char: extern "C" fn(handle: *mut UartHandle, data: u8),
    uart_put_line: extern "C" fn(handle: *mut UartHandle, param: *mut UART_PARAM) -> ErrorCode,
}

impl USART0Driver {
    pub unsafe fn new() -> Self {
        let table_adr = *ROM_API_BASE_ADR;
        let table_usart0 = table_adr + UASRT0_OFFSET;
        let usart0_addr = *(table_usart0 as *const usize);
        USART0Driver {
            base_addr: 1073774592, //use usart0
            uart_get_mem_size: *(usart0_addr as *const extern "C" fn() -> u32),
            uart_setup: *((usart0_addr + 4)
                as *const extern "C" fn(base_addr: u32, ram: *mut u8) -> *mut UartHandle),
            uart_init: *((usart0_addr + 8)
                as *const extern "C" fn(handle: *mut UartHandle, set: *mut UART_CONFIG) -> u32),
            uart_put_char: *((usart0_addr + 16)
                as *const extern "C" fn(handle: *mut UartHandle, data: u8)),
            uart_put_line: *((usart0_addr + 24)
                as *const extern "C" fn(
                    handle: *mut UartHandle,
                    param: *mut UART_PARAM,
                ) -> ErrorCode),
        }
    }

    pub fn get_mem_size(&self) -> u32 {
        (self.uart_get_mem_size)()
    }

    pub fn setup(&self, ram: *mut u8) -> *mut UartHandle {
        (self.uart_setup)(self.base_addr, ram)
    }

    pub fn init(&self, handle: *mut UartHandle, set: *mut UART_CONFIG) -> u32 {
        (self.uart_init)(handle, set)
    }

    pub fn put_char(&self, handle: *mut UartHandle, data: u8) {
        (self.uart_put_char)(handle, data)
    }
    pub fn put_line(&self, handle: *mut UartHandle, param: *mut UART_PARAM) {
        let _err = (self.uart_put_line)(handle, param);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UART_PARAM {
    pub buffer: *const u8,
    pub size: u32,
    pub transfer_mode: u16,
    pub driver_mode: u8,
    pub dma_num: u8,
    pub callback_func_pt: UartCallback,
    pub dma: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UART_CONFIG {
    #[doc = "< System clock in Hz"]
    pub sys_clk_in_hz: u32,
    #[doc = "< Baud rate in Hz"]
    pub baudrate_in_hz: u32,
    #[doc = "< Configuration value */"]
    pub config: u8,
    #[doc = "< Sync mode settings */"]
    pub sync_mod: u8,
    #[doc = "< Errors to be enabled */"]
    pub error_en: u16,
}
