use super::{CVoid, ErrorCode, I2C_OFFSET, ROM_API_BASE_ADR};
pub type I2CHandle = CVoid;
pub type I2CCallback = Option<unsafe extern "C" fn(err_code: u32, n: u32)>;

static mut I2C_DRIVER_MEMORY_BUFFER: [u8; 100] = [0u8; 100];

pub enum Bus {
    I2C0,
    I2C1,
}
pub struct I2CDriver {
    i2c_isr_handler: unsafe extern "C" fn(handle: *const I2CHandle),
    i2c_master_transmit_poll: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_master_receive_poll: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_master_tx_rx_poll: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_master_transmit_intr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_master_receive_intr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_master_tx_rx_intr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_slave_receive_poll: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_slave_transmit_poll: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_slave_receive_intr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_slave_transmit_intr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode,
    i2c_set_slave_addr: unsafe extern "C" fn(
        handle: *const I2CHandle,
        slave_addr_0_3: u32,
        slave_mask_0_3: u32,
    ) -> ErrorCode,
    i2c_get_mem_size: unsafe extern "C" fn() -> u32,
    i2c_setup: unsafe extern "C" fn(i2c_base_addr: u32, start_of_ram: *mut u8) -> *const I2CHandle,
    i2c_set_bitrate: unsafe extern "C" fn(
        handle: *const I2CHandle,
        p_clk_in_hz: u32,
        bitrate_in_bps: u32,
    ) -> ErrorCode,
    i2c_get_firmware_version: unsafe extern "C" fn() -> u32,
    i2c_get_status: unsafe extern "C" fn(handle: *mut I2CHandle) -> ChipI2cMode,
    handle: usize,
    ram: usize,
}

impl I2CDriver {
    pub unsafe fn new(bus: Bus) -> Self {
        let table_adr = *ROM_API_BASE_ADR;
        let table_i2c = table_adr + I2C_OFFSET;
        let i2c_addr = *(table_i2c as *const usize);
        let ram_pointer: *mut u8 = core::ptr::addr_of_mut!(I2C_DRIVER_MEMORY_BUFFER) as *mut u8;
        let setup_fnc = *((i2c_addr + 52)
            as *const unsafe extern "C" fn(
                i2c_base_addr: u32,
                start_of_ram: *mut u8,
            ) -> *mut I2CHandle);
        let handle = match bus {
            Bus::I2C0 => setup_fnc(0x4000_0000, ram_pointer),
            Bus::I2C1 => setup_fnc(0x4002_0000, ram_pointer),
        };
        I2CDriver {
            i2c_isr_handler: *(i2c_addr as *const unsafe extern "C" fn(handle: *const I2CHandle)),
            i2c_master_transmit_poll: *((i2c_addr + 4)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_master_receive_poll: *((i2c_addr + 8)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_master_tx_rx_poll: *((i2c_addr + 12)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_master_transmit_intr: *((i2c_addr + 16)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_master_receive_intr: *((i2c_addr + 20)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_master_tx_rx_intr: *((i2c_addr + 24)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_slave_receive_poll: *((i2c_addr + 28)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_slave_transmit_poll: *((i2c_addr + 32)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_slave_receive_intr: *((i2c_addr + 36)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_slave_transmit_intr: *((i2c_addr + 40)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    param: *mut I2cParam,
                    result: *mut I2cResult,
                ) -> ErrorCode),
            i2c_set_slave_addr: *((i2c_addr + 44)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    slave_addr_0_3: u32,
                    slave_mask_0_3: u32,
                ) -> ErrorCode),
            i2c_get_mem_size: *((i2c_addr + 48) as *const unsafe extern "C" fn() -> u32),
            i2c_setup: *((i2c_addr + 52)
                as *const unsafe extern "C" fn(
                    i2c_base_addr: u32,
                    start_of_ram: *mut u8,
                ) -> *const I2CHandle),
            i2c_set_bitrate: *((i2c_addr + 56)
                as *const unsafe extern "C" fn(
                    handle: *const I2CHandle,
                    p_clk_in_hz: u32,
                    bitrate_in_bps: u32,
                ) -> ErrorCode),
            i2c_get_firmware_version: *((i2c_addr + 60) as *const unsafe extern "C" fn() -> u32),
            i2c_get_status: *((i2c_addr + 64)
                as *const unsafe extern "C" fn(handle: *mut I2CHandle) -> ChipI2cMode),
            handle: handle as usize,
            ram: ram_pointer as usize,
        }
    }
    pub fn get_mem_size(&self) -> u32 {
        unsafe { (self.i2c_get_mem_size)() }
    }
    /*pub fn setup(&self, base_addr: u32, ram: *mut u8) -> *mut I2CHandle {
        unsafe { (self.i2c_setup)(base_addr, ram) }
    }*/
    pub fn get_status(&self) -> ChipI2cMode {
        unsafe { (self.i2c_get_status)(self.handle as *mut _) }
    }
    pub fn set_bitrate(&self, p_clk_hz: u32, bitrate_bps: u32) -> ErrorCode {
        unsafe { (self.i2c_set_bitrate)(self.handle as *const _, p_clk_hz, bitrate_bps) }
    }
    pub fn master_tx_rx_poll(&self, param: *mut I2cParam, result: *mut I2cResult) -> ErrorCode {
        unsafe { (self.i2c_master_tx_rx_poll)(self.handle as *const _, param, result) }
    }
    pub fn master_transmit_poll(
        &self,
        param: *mut I2cParam,
        result: *mut I2cResult,
    ) -> ErrorCode {
        unsafe { (self.i2c_master_transmit_poll)(self.handle as *const _, param, result) }
    }
    pub fn master_receive_poll(&self, param: *mut I2cParam, result: *mut I2cResult) -> ErrorCode {
        unsafe { (self.i2c_master_receive_poll)(self.handle as *const _, param, result) }
    }
    pub fn set_slave_addr(&self, address: u32, mask: u32) -> ErrorCode {
        unsafe { (self.i2c_set_slave_addr)(self.handle as *const _, address, mask) }
    }

    pub fn slave_receive_poll(&self, param: *mut I2cParam, result: *mut I2cResult) -> ErrorCode {
        unsafe { (self.i2c_slave_receive_poll)(self.handle as *const _, param, result) }
    }
    pub fn slave_transmit_poll(&self, param: *mut I2cParam, result: *mut I2cResult) -> ErrorCode {
        unsafe { (self.i2c_slave_transmit_poll)(self.handle as *const _, param, result) }
    }
    pub fn isr_handler(&self) {
        unsafe { (self.i2c_isr_handler)(self.handle as *const _) }
    }
    pub fn slave_receive_intr(&self, param: *mut I2cParam, result: *mut I2cResult) -> ErrorCode {
        unsafe { (self.i2c_slave_receive_intr)(self.handle as *const _, param, result) }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct I2cParam {
    #[doc = "< No. of bytes to send"]
    pub num_bytes_send: u32,
    #[doc = "< No. of bytes to receive"]
    pub num_bytes_rec: u32,
    #[doc = "< Pointer to send buffer"]
    pub buffer_ptr_send: *mut u8,
    #[doc = "< Pointer to receive buffer"]
    pub buffer_ptr_rec: *mut u8,
    #[doc = "< Callback function"]
    pub func_pt: I2CCallback,
    #[doc = "< Stop flag"]
    pub stop_flag: u8,
    pub dummy: [u8; 3usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct I2cResult {
    #[doc = "< No. of bytes sent"]
    pub n_bytes_sent: u32,
    #[doc = "< No. of bytes received"]
    pub n_bytes_recd: u32,
}

#[repr(u32)]
#[doc = "LPC11U6X I2C ROM driver modes enum"]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ChipI2cMode {
    #[doc = "< IDLE state"]
    Idle = 0,
    #[doc = "< Master send state"]
    MasterSend = 1,
    #[doc = "< Master Receive state"]
    MasterReceive = 2,
    #[doc = "< Slave send state"]
    SlaveSend = 3,
    #[doc = "< Slave receive state"]
    SlaveReceive = 4,
}
