

pub struct Adc{
    adc_periph: lpc11u6x_pac::ADC
}

impl Adc{

    pub fn new(periph: lpc11u6x_pac::ADC, sys: &lpc11u6x_pac::SYSCON) -> Self {
        let ret = Adc{adc_periph: periph};
        ret.init(sys);
        ret.calibrate();
        ret
    }
    fn init(&self, sys: &lpc11u6x_pac::SYSCON) {
        sys.pdruncfg.modify(|_r, w| w.adc_pd().powered());
        //crate::dbg::debug("ADC init complete");
    }


    fn calibrate(&self) {
        //let clock: u32 = 500_000;
        //let sys_clock: u32 = crate::clock::get_sys_clk(sys);
        //let divider = (((sys_clock + (clock >> 2)) / clock) - 1)
        //    .try_into()
        //    .expect("Error in ADC Divider Calculation");

        self.adc_periph.ctrl
            .modify(|_r, w| w.cal_mode().set_bit().lpwrmode().bit(false));

        // wait for calibration to finish
        while self.adc_periph.ctrl.read().cal_mode().bit_is_set() {}
        self.adc_periph.ctrl.reset();
        self.adc_periph.trm.modify(|_r, w| w.vrange().low_voltage());
        //crate::dbg::debug("ADC calibration complete");
    }



    pub fn read_channel(&self, channel: usize) -> u16 {
        self.adc_periph.seqa_ctrl
            .modify(|_r, w| w.seqa_ena().disabled().trigpol().bit(true));
        self.adc_periph.seqa_ctrl.modify(|_r, w| unsafe {
            w.channels()
                .bits(1 << channel)
                //.mode()
                //.set_bit()
                .seqa_ena()
                .enabled()
        });
        self.adc_periph.seqa_ctrl.modify(|_r, w| w.start().set_bit());

        while !self.adc_periph.dat[channel].read().datavalid().bit() {
            //crate::dbg::debug("SPIN");
        } // Mark1
        let res = self.adc_periph.dat[channel].read().result().bits();
        ((res as f64) * 0.65811965812) as u16
    }

    pub fn voltage_bytes(&self, channel: usize) -> (u8, u8) {
        let voltage: u16 = self.read_channel(channel);

        //crate::dbg::debug("read_channel:");
        //crate::dbg::debug_number(channel as u32);
        //crate::dbg::debug("result:");
        //crate::dbg::debug_number(voltage as u32);
        ((voltage >> 8) as u8, (voltage % 256) as u8)
    }

}


