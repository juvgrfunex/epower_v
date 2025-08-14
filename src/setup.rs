use lpc11u6x_pac::{generic::Reg, syscon::sysahbclkctrl::SYSAHBCLKCTRL_SPEC};

pub fn enable_clocks(clk_ctrl: &Reg<SYSAHBCLKCTRL_SPEC>) {
    clk_ctrl.modify(|_r, w| {
        w.iocon()
            .enable()
            .i2c0()
            .enable()
            .i2c1()
            .enable()
            .pint()
            .enable()
            .gpio()
            .enable()
            .group0int()
            .enable()
            .usart0()
            .enable()
            .usb()
            .enable()
            .usbsram()
            .enable()
            .adc()
            .enable()
    });
}

pub fn setup_io_pins(iocon: &lpc11u6x_pac::IOCON, gpio: &lpc11u6x_pac::GPIO_PORT) {
    // Shutdown VRM ASAP
    gpio.dir[1].write(|w| w.dirp27().set_bit());
    gpio.dir[2].write(|w| w.dirp2().set_bit());

    //PGOOD NVVDD
    iocon
        .pio0_13()
        .modify(|_r, w| w.mode().inactive_no_pull_do());
    //PGOOD NVVDDS
    iocon.pio1_[10].modify(|_r, w| w.mode().inactive_no_pull_do());

    //adc inputs

    //bits(0) is to set admode which is missing in the PAC/SVD
    iocon
        .pio0_16()
        .modify(|_r, w| unsafe { w.bits(0).mode().inactive_no_pull_do().func().bits(1) }); // Primary output

    iocon
        .pio0_23()
        .modify(|_r, w| unsafe { w.bits(0).mode().inactive_no_pull_do().func().bits(1) }); // Secondary output

    iocon
        .pio0_12()
        .modify(|_r, w| unsafe { w.bits(0).mode().inactive_no_pull_do().func().bits(2) }); // 12v

    iocon.pio1_[29]
        .modify(|_r, w| unsafe { w.bits(0).mode().inactive_no_pull_do().func().bits(4) }); // 5v

    iocon
        .pio0_11()
        .modify(|_r, w| unsafe { w.bits(0).mode().inactive_no_pull_do().func().bits(2) }); // 3v

    // usart0
    iocon.pio0_18().modify(|_r, w| unsafe { w.func().bits(1) });
    iocon.pio0_19().modify(|_r, w| unsafe { w.func().bits(1) });

    //i2c0
    iocon
        .pio0_4
        .write(|w| unsafe { w.func().bits(1 | (1 << 7)) });
    iocon
        .pio0_5
        .write(|w| unsafe { w.func().bits(1 | (1 << 7)) });

    //i2c1
    iocon
        .pio0_7()
        .write(|w| unsafe { w.func().bits(3).od().enable().mode().inactive_no_pull_do() });
    iocon.pio1_[24]
        .write(|w| unsafe { w.func().bits(2).od().enabled().mode().inactive_no_pull_do() });

    // disable RGB LED
    gpio.dir[1].modify(|_r, w| w.dirp23().set_bit());
    gpio.dir[2].modify(|_r, w| w.dirp7().set_bit());
    gpio.dir[0].write(|w| w.dirp6().set_bit());
}

#[allow(dead_code)]
pub fn restore_gpio_after_bootloader(periph: &lpc11u6x_pac::Peripherals) {
    //! Reset the registers modified by the bootloader back to default bootup values.
    //!
    //! Unused since the bootloader is removed with this version.
    periph.I2C0.conset.reset();
    periph.I2C0.dat.reset();
    periph.I2C0.sclh.reset();
    periph.I2C0.scll.reset();
    periph.I2C0.conclr.reset();

    periph.CT16B0.ir.reset();
    periph.CT16B0.tcr.reset(); //f
    periph.CT16B0.tc.reset();
    periph.CT16B0.pr.reset();
    periph.CT16B0.pc.reset();
    periph.CT16B0.mcr.reset();
    periph.CT16B0.mr[1].reset();

    periph.CT16B1.ir.reset();
    periph.CT16B1.tcr.reset(); //g
    periph.CT16B1.tc.reset();
    periph.CT16B1.pr.reset();
    periph.CT16B1.pc.reset();
    periph.CT16B1.mcr.reset();
    periph.CT16B1.mr[2].reset();

    periph.I2C1.conset.reset();
    periph.I2C1.adr0.reset();
    periph.I2C1.conclr.reset();
    periph.I2C1.adr1().reset();

    periph.GPIO_PORT.b[2].reset();
    periph.GPIO_PORT.b[21].reset();
    periph.GPIO_PORT.b[39].reset();
    periph.GPIO_PORT.b[51].reset();
    periph.GPIO_PORT.b[52].reset();
    periph.GPIO_PORT.b[58].reset();
    periph.GPIO_PORT.b[69].reset();
    periph.GPIO_PORT.b[70].reset();
}
