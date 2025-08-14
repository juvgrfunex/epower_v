pub fn init(sys: &lpc11u6x_pac::SYSCON, uart: &lpc11u6x_pac::USART0) {
    unsafe { sys.usart0clkdiv.write(|w| w.bits(1)) };
    sys.sysahbclkctrl.modify(|_r, w| w.usart0().enable());
    unsafe { uart.fcr().write(|w| w.bits(0x7)) };
    unsafe { uart.fdr.write(|w| w.bits(0x10)) };
}

pub fn de_init(sys: &lpc11u6x_pac::SYSCON) {
    sys.sysahbclkctrl.modify(|_r, w| w.usart0().disable());
}

pub fn send_bytes(uart: &lpc11u6x_pac::USART0, data: &[u8]) -> usize {
    let mut sent: usize = 0;

    while sent < data.len() && (uart.lsr.read().bits() & (1 << 5) != 0) {
        unsafe { uart.thr().write(|w| w.bits(data[sent] as u32)) };
        sent += 1;
    }
    sent
}

pub fn chip_uart0_set_baud(sys: &lpc11u6x_pac::SYSCON, uart: &lpc11u6x_pac::USART0, baudrate: u32) -> u32 {
    // c code set clock div here
    let clkin: u32 = crate::clock::get_main_clk(sys);
    let div: u32 = clkin / (baudrate * 16);
    let divh: u32 = div / 256;
    let divl: u32 = div - (divh * 256);
    unsafe {
        uart.lcr.modify(|r, w| w.bits(r.bits() | 0b10000000));
        uart.dll().write(|w| w.bits(divl));
        uart.dlm().write(|w| w.bits(divh));
        uart.lcr.modify(|r, w| w.bits(r.bits() & 0b01111111));
    }
    clkin / (div * 16)
}

pub fn config(uart: &lpc11u6x_pac::USART0) {
    unsafe { uart.lcr.write(|w| w.bits(1 | 1 << 1)) }
}
pub fn setfup_fifos(uart: &lpc11u6x_pac::USART0) {
    unsafe { uart.fcr().write(|w| w.bits(1 | (2 << 6))) }
}

pub fn tx_enable(uart: &lpc11u6x_pac::USART0) {
    unsafe { uart.ter.write(|w| w.bits(1 << 7)) }
}
