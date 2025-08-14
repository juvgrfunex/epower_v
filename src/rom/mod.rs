pub const ROM_API_BASE_ADR: *const usize = 0x1FFF_1FF8 as *const usize;
pub const UASRT0_OFFSET: usize = 0x2C;
pub const I2C_OFFSET: usize = 0x14;
pub const IAP_ENTRY: usize = 0x1FFF1FF1;
#[derive(Debug, Copy, Clone)]
pub enum CVoid {}

pub mod eeprom;
pub mod i2c;
pub mod uart;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum IapStatus {
    CmdSuccess,
    InvalidCommand,
    SrcAddrError,
    DstAddrError,
    SrcAddrNotMapped,
    DstAddrNotMapped,
    CountError,
    InvalidSector,
    SectorNotBlank,
    SectorNotPreparedForWriteOperation,
    CompareError,
    Busy,
    Unknown,
}

impl From<u32> for IapStatus {
    fn from(id: u32) -> Self {
        match id {
            0 => Self::CmdSuccess,
            1 => Self::InvalidCommand,
            2 => Self::SrcAddrError,
            3 => Self::DstAddrError,
            4 => Self::SrcAddrNotMapped,
            5 => Self::DstAddrNotMapped,
            6 => Self::CountError,
            7 => Self::InvalidSector,
            8 => Self::SectorNotBlank,
            9 => Self::SectorNotPreparedForWriteOperation,
            10 => Self::CompareError,
            11 => Self::Busy,
            _ => Self::Unknown,
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum ErrorCode {
    #[doc = "< enum value returned on Success"]
    LPC_OK = 0,
    #[doc = "< enum value returned on general failure"]
    ERR_FAILED = -1,
    #[doc = "< enum value returned on general timeout"]
    ERR_TIME_OUT = -2,
    #[doc = "< enum value returned when resource is busy"]
    ERR_BUSY = -3,
    ERR_ISP_INVALID_COMMAND = 1,
    ERR_ISP_SRC_ADDR_ERROR = 2,
    ERR_ISP_DST_ADDR_ERROR = 3,
    ERR_ISP_SRC_ADDR_NOT_MAPPED = 4,
    ERR_ISP_DST_ADDR_NOT_MAPPED = 5,
    ERR_ISP_COUNT_ERROR = 6,
    ERR_ISP_INVALID_SECTOR = 7,
    ERR_ISP_SECTOR_NOT_BLANK = 8,
    ERR_ISP_SECTOR_NOT_PREPARED_FOR_WRITE_OPERATION = 9,
    ERR_ISP_COMPARE_ERROR = 10,
    ERR_ISP_BUSY = 11,
    ERR_ISP_PARAM_ERROR = 12,
    ERR_ISP_ADDR_ERROR = 13,
    ERR_ISP_ADDR_NOT_MAPPED = 14,
    ERR_ISP_CMD_LOCKED = 15,
    ERR_ISP_INVALID_CODE = 16,
    ERR_ISP_INVALID_BAUD_RATE = 17,
    ERR_ISP_INVALID_STOP_BIT = 18,
    ERR_ISP_CODE_READ_PROTECTION_ENABLED = 19,
    ERR_API_BASE = 65536,
    #[doc = "< Invalid parameters"]
    ERR_API_INVALID_PARAMS = 65537,
    #[doc = "< PARAM1 is invalid"]
    ERR_API_INVALID_PARAM1 = 65538,
    #[doc = "< PARAM2 is invalid"]
    ERR_API_INVALID_PARAM2 = 65539,
    #[doc = "< PARAM3 is invalid"]
    ERR_API_INVALID_PARAM3 = 65540,
    #[doc = "< API is called before module init"]
    ERR_API_MOD_INIT = 65541,
    ERR_SPIFI_BASE = 131072,
    ERR_SPIFI_DEVICE_ERROR = 131073,
    ERR_SPIFI_INTERNAL_ERROR = 131074,
    ERR_SPIFI_TIMEOUT = 131075,
    ERR_SPIFI_OPERAND_ERROR = 131076,
    ERR_SPIFI_STATUS_PROBLEM = 131077,
    ERR_SPIFI_UNKNOWN_EXT = 131078,
    ERR_SPIFI_UNKNOWN_ID = 131079,
    ERR_SPIFI_UNKNOWN_TYPE = 131080,
    ERR_SPIFI_UNKNOWN_MFG = 131081,
    ERR_SEC_BASE = 196608,
    ERR_SEC_AES_WRONG_CMD = 196609,
    ERR_SEC_AES_NOT_SUPPORTED = 196610,
    ERR_SEC_AES_KEY_ALREADY_PROGRAMMED = 196611,
    ERR_USBD_BASE = 262144,
    #[doc = "< invalid request"]
    ERR_USBD_INVALID_REQ = 262145,
    #[doc = "< Callback did not process the event"]
    ERR_USBD_UNHANDLED = 262146,
    #[doc = "< Stall the endpoint on which the call back is called"]
    ERR_USBD_STALL = 262147,
    #[doc = "< Send ZLP packet on the endpoint on which the call back is called"]
    ERR_USBD_SEND_ZLP = 262148,
    #[doc = "< Send data packet on the endpoint on which the call back is called"]
    ERR_USBD_SEND_DATA = 262149,
    #[doc = "< Bad descriptor"]
    ERR_USBD_BAD_DESC = 262150,
    #[doc = "< Bad config descriptor"]
    ERR_USBD_BAD_CFG_DESC = 262151,
    #[doc = "< Bad interface descriptor"]
    ERR_USBD_BAD_INTF_DESC = 262152,
    #[doc = "< Bad endpoint descriptor"]
    ERR_USBD_BAD_EP_DESC = 262153,
    #[doc = "< Bad alignment of buffer passed."]
    ERR_USBD_BAD_MEM_BUF = 262154,
    #[doc = "< Too many class handlers."]
    ERR_USBD_TOO_MANY_CLASS_HDLR = 262155,
    ERR_CGU_BASE = 327680,
    ERR_CGU_NOT_IMPL = 327681,
    ERR_CGU_INVALID_PARAM = 327682,
    ERR_CGU_INVALID_SLICE = 327683,
    ERR_CGU_OUTPUT_GEN = 327684,
    ERR_CGU_DIV_SRC = 327685,
    ERR_CGU_DIV_VAL = 327686,
    ERR_CGU_SRC = 327687,
    ERR_I2C_BASE = 393216,
    ERR_I2C_NAK = 393217,
    ERR_I2C_BUFFER_OVERFLOW = 393218,
    ERR_I2C_BYTE_COUNT_ERR = 393219,
    ERR_I2C_LOSS_OF_ARBRITRATION = 393220,
    ERR_I2C_SLAVE_NOT_ADDRESSED = 393221,
    ERR_I2C_LOSS_OF_ARBRITRATION_NAK_BIT = 393222,
    ERR_I2C_GENERAL_FAILURE = 393223,
    ERR_I2C_REGS_SET_TO_DEFAULT = 393224,
    ERR_I2C_TIMEOUT = 393225,
    ERR_I2C_BUFFER_UNDERFLOW = 393226,
    ERR_I2C_UNKNOWN_MODE = 393227,
    ERR_I2C_PARAM = 393228,
    ERR_I2C_DMA_SETUP = 393229,
    ERR_I2C_BUS_ERROR = 393230,
    ERR_UART_BASE = 524288,
    #[doc = "< Receive is busy"]
    ERR_UART_RXD_BUSY = 524289,
    #[doc = "< Transmit is busy"]
    ERR_UART_TXD_BUSY = 524290,
    #[doc = "< Overrun, Frame, Parity , Receive Noise error"]
    ERR_UART_OVERRUN_FRAME_PARITY_NOISE = 524291,
    #[doc = "< Underrun"]
    ERR_UART_UNDERRUN = 524292,
    #[doc = "< Parameter error"]
    ERR_UART_PARAM = 524293,
    ERR_DMA_BASE = 851968,
    ERR_DMA_ERROR_INT = 851969,
    ERR_DMA_CHANNEL_NUMBER = 851970,
    ERR_DMA_CHANNEL_DISABLED = 851971,
    ERR_DMA_BUSY = 851972,
    ERR_DMA_NOT_ALIGNMENT = 851973,
    ERR_DMA_PING_PONG_EN = 851974,
    ERR_DMA_CHANNEL_VALID_PENDING = 851975,
    ERR_SPI_BASE = 917504,
    ERR_SPI_RXOVERRUN = 917505,
    ERR_SPI_TXUNDERRUN = 917506,
    ERR_SPI_SELNASSERT = 917507,
    ERR_SPI_SELNDEASSERT = 917508,
    ERR_SPI_CLKSTALL = 917509,
    ERR_SPI_PARAM = 917510,
    ERR_SPI_INVALID_LENGTH = 917511,
    ERR_ADC_BASE = 983040,
    ERR_ADC_OVERRUN = 983041,
    ERR_ADC_INVALID_CHANNEL = 983042,
    ERR_ADC_INVALID_SEQUENCE = 983043,
    ERR_ADC_INVALID_SETUP = 983044,
    ERR_ADC_PARAM = 983045,
    ERR_ADC_INVALID_LENGTH = 983046,
    ERR_ADC_NO_POWER = 983047,
}
