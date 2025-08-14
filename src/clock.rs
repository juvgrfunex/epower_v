use core::convert::TryFrom;
pub fn get_main_clk(sys: &lpc11u6x_pac::SYSCON) -> u32 {
    let temp = sys.mainclksel.read().bits() & 0x3;
    match MainClockSrc::try_from(temp) {
        Ok(MainClockSrc::Irc) => 12000000,
        Ok(MainClockSrc::PllIn) => get_pll_freq(sys.syspllctrl.read().bits(), get_sys_pll_clk(sys)),
        Ok(MainClockSrc::WdtOsc) => sys.wdtoscctrl.read().bits(),
        Ok(MainClockSrc::PllOut) => 48000000, /*get_pll(syscon.usbpllctrl.read().bits(), inputRate)*/
        Err(_) => 0,
    }
}

pub fn get_sys_pll_clk(sys: &lpc11u6x_pac::SYSCON) -> u32 {
    match SysPllClockSrc::try_from(sys.syspllclksel.read().bits() & 0x03).expect("This try_from can not fail when called with 2bit value") {
        SysPllClockSrc::Irc => unimplemented!(),
        SysPllClockSrc::Main => 12000000,
        SysPllClockSrc::Rsvd => unimplemented!(),
        SysPllClockSrc::RTC32k => unimplemented!(),
    }
}
pub fn get_sys_clk(sys: &lpc11u6x_pac::SYSCON) -> u32 {
    let main = get_main_clk(sys);
    let div = sys.sysahbclkdiv.read().bits();
    main / div
}

pub fn set_48_mhz(sys: &lpc11u6x_pac::SYSCON) {
    sys.pdruncfg.modify(|_r, w| w.syspll_pd().bit(false));

    unsafe {
        sys.syspllclksel.write(|w| w.bits(0));
        sys.syspllclkuen.write(|w| w.bits(0));
        sys.syspllclkuen.write(|w| w.bits(1));
        sys.syspllctrl.write(|w| w.bits(3));
        while sys.syspllstat.read().bits() == 0 {
            continue;
        }
        sys.mainclksel.write(|w| w.bits(3));
        sys.mainclkuen.write(|w| w.bits(0));
        sys.mainclkuen.write(|w| w.bits(1));
    }
}
enum MainClockSrc {
    Irc = 0,
    PllIn = 1,
    WdtOsc = 2,
    PllOut = 3,
}

enum SysPllClockSrc {
    Irc = 0,
    Main = 1,
    Rsvd = 2,
    RTC32k = 3,
}

impl TryFrom<u32> for MainClockSrc {
    type Error = u8;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MainClockSrc::Irc),
            1 => Ok(MainClockSrc::PllIn),
            2 => Ok(MainClockSrc::WdtOsc),
            3 => Ok(MainClockSrc::PllOut),
            _ => Err(0),
        }
    }
}

impl TryFrom<u32> for SysPllClockSrc {
    type Error = u8;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SysPllClockSrc::Irc),
            1 => Ok(SysPllClockSrc::Main),
            2 => Ok(SysPllClockSrc::Rsvd),
            3 => Ok(SysPllClockSrc::RTC32k),
            _ => Err(0),
        }
    }
}

fn get_pll_freq(pllreg: u32, input_rate: u32) -> u32 {
    ((pllreg & 0x1F) + 1) * input_rate
}
