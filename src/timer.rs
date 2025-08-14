pub enum TimerSel {
    CT16B0,
    CT16B1,
    CT32B0,
    CT32B1,
}

pub struct Timer<T> {
    periph: T,
}

pub fn enable_timer<T>(sys: &lpc11u6x_pac::SYSCON, timer: TimerSel, periph: T) -> Timer<T> {
    match timer {
        TimerSel::CT16B0 => sys.sysahbclkctrl.modify(|_r, w| w.ct16b0().enable()),
        TimerSel::CT16B1 => sys.sysahbclkctrl.modify(|_r, w| w.ct16b1().enable()),
        TimerSel::CT32B0 => sys.sysahbclkctrl.modify(|_r, w| w.ct32b0().enable()),
        TimerSel::CT32B1 => sys.sysahbclkctrl.modify(|_r, w| w.ct32b1().enable()),
    }
    Timer { periph }
}

impl Timer<lpc11u6x_pac::CT16B0> {
    pub fn reset(&self) {
        let reg = self.periph.tcr.read().bits();
        unsafe {
            self.periph.tcr.write(|w| w.bits(0));
            self.periph.tc.write(|w| w.bits(1));
            self.periph.tcr.write(|w| w.bits(2));
            while self.periph.tc.read().bits() != 0 {}
            self.periph.tcr.write(|w| w.bits(reg));
        }
    }

    pub fn set_prescale(&self, value: u16) {
        unsafe { self.periph.pr.write(|w| w.pcval().bits(value)) }
    }

    pub fn set_match(&self, matchnum: usize, matchval: u16) {
        unsafe { self.periph.mr[matchnum].write(|w| w.match_().bits(matchval)) };
    }
    pub fn int_on_match(&self, matchnum: usize) {
        let value = 1 << (matchnum * 3);
        unsafe { self.periph.mcr.modify(|r, w| w.bits(r.bits() | (value))) }
    }

    pub fn reset_on_match(&self, matchnum: usize) {
        let value = 1 << ((matchnum * 3) + 1);
        unsafe { self.periph.mcr.modify(|r, w| w.bits(r.bits() | (value))) }
    }
    pub fn start(&self) {
        unsafe { self.periph.tcr.write(|w| w.bits(1)) };
    }
    pub fn clear_int(&self) {
        unsafe {
            self.periph.ir.write(|w| w.bits(2));
        }
    }
}

impl Timer<lpc11u6x_pac::CT16B1> {
    pub fn reset(&self) {
        let reg = self.periph.tcr.read().bits();
        unsafe {
            self.periph.tcr.write(|w| w.bits(0));
            self.periph.tc.write(|w| w.bits(1));
            self.periph.tcr.write(|w| w.bits(2));
            while self.periph.tc.read().bits() != 0 {}
            self.periph.tcr.write(|w| w.bits(reg));
        }
    }

    pub fn set_prescale(&self, value: u16) {
        unsafe { self.periph.pr.write(|w| w.pcval().bits(value)) }
    }

    pub fn set_match(&self, matchnum: usize, matchval: u16) {
        unsafe { self.periph.mr[matchnum].write(|w| w.match_().bits(matchval)) };
    }
    pub fn int_on_match(&self, matchnum: usize) {
        let value = 1 << (matchnum * 3);
        unsafe { self.periph.mcr.modify(|r, w| w.bits(r.bits() | (value))) }
    }

    pub fn reset_on_match(&self, matchnum: usize) {
        let value = 1 << ((matchnum * 3) + 1);
        unsafe { self.periph.mcr.modify(|r, w| w.bits(r.bits() | (value))) }
    }
    pub fn start(&self) {
        unsafe { self.periph.tcr.write(|w| w.bits(1)) };
    }
    pub fn clear_int(&self) {
        unsafe {
            self.periph.ir.write(|w| w.bits(2));
        }
    }
}
