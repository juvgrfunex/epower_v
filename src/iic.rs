use crate::disp;
use core::hint::unreachable_unchecked;

pub(crate) const IR3595_ADDR: u8 = 0x08;
pub(crate) const IR3595_ADDR_SHIFTED: u8 = IR3595_ADDR << 1;
pub(crate) const EPOWER_ADDR: u8 = 0x0E;
pub(crate) const EPOWER_ADDR_SHIFTED: u8 = EPOWER_ADDR << 1;

mod i2c_driver_states {
    /// Own SLA+W has been received; ACK has been returned.
    pub const SLAVE_WRITE_RECIEVED_AND_ACKED: u32 = 0x60;

    // Previously addressed with own SLV address; DATA has been received; ACK has been returned
    pub const RECIEVED_DATA_AND_ACKED: u32 = 0x80;

    // A STOP condition or Repeated START condition has been received while still addressed as SLV/REC or SLV/TRX
    pub const RECIEVED_STOP_OR_REPEAT: u32 = 0xA0;

    // Own SLA+R has been received; ACK has been returned.
    pub const SLAVE_READ_RECIEVED_AND_ACKED: u32 = 0xA8;

    // Data byte in DAT has been transmitted; ACK has been received.
    pub const SEND_DATA_AND_RECIEVED_ACK: u32 = 0xB8;

    // Data byte in DAT has been transmitted; NOT ACK has been received.
    pub const SEND_DATA_AND_RECIEVED_NOT_ACK: u32 = 0xC0;
}

pub fn init_slave_recv(periph: &lpc11u6x_pac::I2C1) {
    unsafe { periph.adr0.write(|w| w.address().bits(IR3595_ADDR)) };
    unsafe { periph.adr1().write(|w| w.address().bits(EPOWER_ADDR)) };
    periph.conset.write(|w| {
        w.i2en()
            .set_bit()
            .sta()
            .clear_bit()
            .sto()
            .clear_bit()
            .si()
            .clear_bit()
            .aa()
            .set_bit()
    });
}

enum I2C1Mode {
    EpowerRead,
    EpowerWrite,
    Ir3595Read,
    IR3595Write,
    Idle,
}

pub struct I2C1State {
    periph: lpc11u6x_pac::I2C1,
    recieve_buffer: [u8; 8],
    recieve_buffer_index: usize,
    mode: I2C1Mode,
    send_buffer: [u8; 4],
    send_buffer_index: usize,
}

impl I2C1State {
    pub fn new(i2c1: lpc11u6x_pac::I2C1) -> Self {
        I2C1State {
            periph: i2c1,
            recieve_buffer: [0; 8],
            recieve_buffer_index: 0,
            mode: I2C1Mode::Idle,
            send_buffer: [0u8; 4],
            send_buffer_index: 0,
        }
    }

    fn monitoring_read(&mut self, cmd: u8, adc: &crate::adc::Adc) {
        let channel = match cmd {
            0 => 2,
            1 => 1,
            2 => 8,
            3 => 10,
            4 => 9,
            _ => {
                return;
            }
        };
        unsafe {
            let (higher, lower) = adc.voltage_bytes(channel);
            self.send_buffer[0] = lower;
            self.periph.dat.write(|w| w.bits(higher as u32));
        }
        self.recieve_buffer_index = 1;
    }

    fn constants_read(&mut self, cmd: u8) {
        match cmd {
            1 => unsafe {
                self.periph
                    .dat
                    .write(|w| w.bits(crate::FIRMWARE_VERSION as u32))
            },
            2 => {
                let uid = crate::rom::eeprom::get_uid().to_be_bytes();
                self.send_buffer = uid;
                self.send_buffer_index = 2;
                unsafe {
                    self.periph
                        .dat
                        .write(|w| w.bits(self.send_buffer[3] as u32))
                };
            }
            _ => {}
        }
    }

    fn settings_read(&mut self, cmd: u8, settings: &crate::settings::Settings) {
        let value = settings.get_setting_value(cmd);
        unsafe { self.periph.dat.write(|w| w.bits(value as u32)) }
    }

    fn settings_write(
        &mut self,
        settings: &mut crate::settings::Settings,
        display: &mut disp::Display,
        gpio: &lpc11u6x_pac::GPIO_PORT,
    ) {
        let cmd = self.recieve_buffer[0];
        let value = self.recieve_buffer[1];
        if (64..128).contains(&cmd) {
            let setting_id = cmd - 64;
            settings.set_setting_value(setting_id, value, display, gpio);
        }

        self.recieve_buffer_index = 0;
    }

    fn handle_epower_read(&mut self, settings: &crate::settings::Settings, adc: &crate::adc::Adc) {
        if self.send_buffer_index > 0 {
            unsafe {
                self.periph
                    .dat
                    .write(|w| w.bits(self.send_buffer[self.send_buffer_index] as u32))
            }
            self.send_buffer_index -= 1;
        } else if self.recieve_buffer_index == 1 {
            let cmd = self.recieve_buffer[0];
            match cmd {
                0..=63 => {
                    self.constants_read(cmd);
                }
                64..=127 => {
                    self.settings_read(cmd - 64, settings);
                }
                128..=250 => {
                    self.monitoring_read(cmd - 128, adc);
                }
                251..=255 => {
                    unsafe { self.periph.dat.write(|w| w.bits(0xEE_u32)) };
                }
            }
            self.recieve_buffer_index = 0;
        } else {
            unsafe { self.periph.dat.write(|w| w.bits(0xEE_u32)) };
        }

        self.periph.conset.write(|w| w.aa().set_bit());
    }

    fn ack_and_clear_int(&self) {
        self.periph.conset.write(|w| w.aa().set_bit());
        self.periph.conclr.write(|w| w.sic().set_bit());
    }

    pub fn handle_state(
        &mut self,
        settings: &mut crate::settings::Settings,
        vrm_i2c_driver: &crate::rom::i2c::I2CDriver,
        display: &mut disp::Display,
        adc: &crate::adc::Adc,
        gpio: &lpc11u6x_pac::GPIO_PORT,
    ) {
        let stat = self.periph.stat.read().bits();

        match stat {
            i2c_driver_states::SLAVE_WRITE_RECIEVED_AND_ACKED => {
                let raw_addr_byte = self.periph.dat.read().bits() as u8;
                let addr = raw_addr_byte >> 1;
                self.mode = match addr {
                    0x08 => I2C1Mode::IR3595Write,
                    0x0E => I2C1Mode::EpowerWrite,
                    _ => I2C1Mode::Idle,
                };
                self.recieve_buffer_index = 0;
                self.ack_and_clear_int();
            }

            i2c_driver_states::RECIEVED_DATA_AND_ACKED => match self.mode {
                I2C1Mode::EpowerWrite | I2C1Mode::IR3595Write => {
                    self.recieve_buffer[self.recieve_buffer_index] =
                        self.periph.dat.read().bits() as u8;
                    self.recieve_buffer_index += 1;
                    self.ack_and_clear_int();
                }
                I2C1Mode::Idle | I2C1Mode::Ir3595Read | I2C1Mode::EpowerRead => {
                    self.periph.conclr.write(|w| w.sic().set_bit());
                }
            },

            i2c_driver_states::RECIEVED_STOP_OR_REPEAT => {
                match self.mode {
                    I2C1Mode::Ir3595Read | I2C1Mode::EpowerRead => {
                        self.recieve_buffer_index = 0;
                        self.send_buffer_index = 0;
                    }
                    I2C1Mode::IR3595Write => {
                        if self.recieve_buffer_index == 2 {
                            crate::vrm::write_reg(
                                vrm_i2c_driver,
                                self.recieve_buffer[0],
                                self.recieve_buffer[1],
                            );
                            self.recieve_buffer_index = 0;
                            self.send_buffer_index = 0;
                        }
                    }
                    I2C1Mode::EpowerWrite => {
                        if self.recieve_buffer_index == 2 {
                            self.settings_write(settings, display, gpio);
                        }
                    }
                    I2C1Mode::Idle => {}
                }
                self.mode = I2C1Mode::Idle;
                self.ack_and_clear_int()
            }

            i2c_driver_states::SLAVE_READ_RECIEVED_AND_ACKED => {
                let raw_addr_byte = self.periph.dat.read().bits() as u8;
                let addr = raw_addr_byte >> 1;
                self.mode = match addr {
                    0x08 => I2C1Mode::Ir3595Read,
                    0x0E => I2C1Mode::EpowerRead,
                    _ => I2C1Mode::Idle,
                };

                match self.mode {
                    I2C1Mode::EpowerRead => {
                        self.handle_epower_read(settings, adc);
                    }
                    I2C1Mode::Ir3595Read => {
                        if self.recieve_buffer_index == 0 {
                            unsafe { self.periph.dat.write(|w| w.bits(0xFF_u32)) };
                        } else if self.recieve_buffer_index == 1 {
                            if let Some(r) =
                                crate::vrm::read_reg(vrm_i2c_driver, self.recieve_buffer[0])
                            {
                                unsafe { self.periph.dat.write(|w| w.bits(r as u32)) };
                            }

                            self.recieve_buffer_index = 0;
                        } else {
                            self.recieve_buffer_index = 0;
                        }
                    }
                    I2C1Mode::IR3595Write | I2C1Mode::EpowerWrite => unsafe {
                        unreachable_unchecked()
                    },
                    I2C1Mode::Idle => {}
                }
                self.ack_and_clear_int();
            }

            i2c_driver_states::SEND_DATA_AND_RECIEVED_ACK => {
                match self.mode {
                    I2C1Mode::Ir3595Read => {
                        if let Some(value) = crate::vrm::read_raw(vrm_i2c_driver) {
                            unsafe { self.periph.dat.write(|w| w.bits(value as u32)) };
                        }
                    }
                    I2C1Mode::EpowerRead => {
                        unsafe {
                            self.periph
                                .dat
                                .write(|w| w.bits(self.send_buffer[self.send_buffer_index] as u32))
                        };
                        if self.send_buffer_index > 0 {
                            self.send_buffer_index -= 1;
                        }
                    }
                    _ => {
                        unsafe { self.periph.dat.write(|w| w.bits(0xEE)) };
                    }
                }

                self.ack_and_clear_int();
            }

            i2c_driver_states::SEND_DATA_AND_RECIEVED_NOT_ACK => {
                self.send_buffer_index = 0;
                self.recieve_buffer_index = 0;
                self.ack_and_clear_int();
            }
            _status_code => {
                // unimplemented state
                // TODO: implement arbitration lost states
                // 0x68
                self.periph
                    .conset
                    .write(|w| w.sta().set_bit().aa().set_bit());
                self.periph.conclr.write(|w| w.sic().set_bit());
            }
        }
        //self.periph.conclr.write(|w| w.sic().set_bit());
    }
}
