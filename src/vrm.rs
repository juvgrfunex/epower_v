use crate::i2c::{I2CDriver, I2cParam, I2cResult};
use crate::iic::IR3595_ADDR_SHIFTED;
use crate::rom::ErrorCode;

pub enum DeviceMode {
    Nvidia,
    InternationalBusinessMachine,
    Invalid,
}

pub fn read_reg_2byte(i2c: &I2CDriver, reg_addr: u8) -> Option<u16> {
    let mut send_buffer = [IR3595_ADDR_SHIFTED, reg_addr, 0];
    let mut recieve_buffer = [(IR3595_ADDR_SHIFTED) | 1, 0, 0];
    let mut param = I2cParam {
        num_bytes_rec: 2,
        num_bytes_send: 2,
        buffer_ptr_rec: recieve_buffer.as_mut_ptr(),
        buffer_ptr_send: send_buffer.as_mut_ptr(),
        func_pt: None,
        stop_flag: 1,
        dummy: [0, 0, 0],
    };
    let result = &mut I2cResult {
        n_bytes_recd: 0,
        n_bytes_sent: 0,
    };
    if i2c.master_tx_rx_poll(&mut param, result) == ErrorCode::LPC_OK {
        let reg_value = ((recieve_buffer[0] as u16) << 8) | recieve_buffer[1] as u16;
        Some(reg_value)
    } else {
        None
    }
}

pub fn read_reg(i2c: &I2CDriver, reg_addr: u8) -> Option<u8> {
    let mut send_buffer = [IR3595_ADDR_SHIFTED, reg_addr, 0];
    let mut recieve_buffer = [(IR3595_ADDR_SHIFTED) | 1, 0];
    let mut param = I2cParam {
        num_bytes_rec: 1,
        num_bytes_send: 2,
        buffer_ptr_rec: recieve_buffer.as_mut_ptr(),
        buffer_ptr_send: send_buffer.as_mut_ptr(),
        func_pt: None,
        stop_flag: 1,
        dummy: [0, 0, 0],
    };
    let result = &mut I2cResult {
        n_bytes_recd: 0,
        n_bytes_sent: 0,
    };
    if i2c.master_tx_rx_poll(&mut param, result) == ErrorCode::LPC_OK {
        Some(recieve_buffer[0])
    } else {
        None
    }
}

pub fn write_reg(i2c: &I2CDriver, reg_addr: u8, data: u8) {
    let mut send_buffer = [IR3595_ADDR_SHIFTED, reg_addr, data];
    let mut recieve_buffer = [(IR3595_ADDR_SHIFTED) | 1, 0];
    let mut param = I2cParam {
        num_bytes_rec: 0,
        num_bytes_send: 3,
        buffer_ptr_rec: recieve_buffer.as_mut_ptr(),
        buffer_ptr_send: send_buffer.as_mut_ptr(),
        func_pt: None,
        stop_flag: 1,
        dummy: [0, 0, 0],
    };
    let result = &mut I2cResult {
        n_bytes_recd: 0,
        n_bytes_sent: 0,
    };
    i2c.master_transmit_poll(&mut param, result);
}

pub fn read_raw(i2c: &I2CDriver) -> Option<u8> {
    let mut recieve_buffer = [(IR3595_ADDR_SHIFTED) | 1, 0];
    let mut param = I2cParam {
        num_bytes_rec: 1,
        num_bytes_send: 0,
        buffer_ptr_rec: recieve_buffer.as_mut_ptr(),
        buffer_ptr_send: core::ptr::null_mut::<u8>(),
        func_pt: None,
        stop_flag: 1,
        dummy: [0, 0, 0],
    };
    let result = &mut I2cResult {
        n_bytes_recd: 0,
        n_bytes_sent: 0,
    };
    if i2c.master_receive_poll(&mut param, result) == ErrorCode::LPC_OK {
        Some(recieve_buffer[0])
    } else {
        None
    }
}

pub fn enable_l2_en(i2c: &I2CDriver) {
    write_reg(i2c, 0x50, 192);
}
pub fn read_en_cfg(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0x50)
}
pub fn read_voltage_l1(i2c: &I2CDriver) -> Option<f32> {
    Some(read_reg_2byte(i2c, 0x9A)? as f32 * 0.488)
}
pub fn read_voltage_l2(i2c: &I2CDriver) -> Option<f32> {
    Some(read_reg(i2c, 0x9C)? as f32 * 15.625)
}

pub fn read_id(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0xFB)
}
pub fn read_sillicon_version(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0xFA)
}
pub fn read_mode(i2c: &I2CDriver) -> Option<DeviceMode> {
    Some(match read_reg(i2c, 0x1C)? & 0b01100000 {
        64 => DeviceMode::Nvidia,
        96 => DeviceMode::InternationalBusinessMachine,
        _ => DeviceMode::Invalid,
    })
}
pub fn read_vr_hot(i2c: &I2CDriver) -> Option<u8> {
    let raw = read_reg(i2c, 0x3A)?;
    Some(((raw & 0b11111100) >> 2) + 64)
}
pub fn read_temp_l1(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0x9D)
}

pub fn read_temp_l2(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0x9E)
}

// always reads 0 on low current
pub fn read_current_l1(i2c: &I2CDriver) -> Option<u16> {
    let factor = match read_reg(i2c, 0x5E)? & 0b00001000 {
        0 => 2_f32,
        8 => 0.25,
        _ => unreachable!(),
    };
    let raw_current = read_reg(i2c, 0x94)?;
    Some((raw_current as f32 * factor) as u16)
}

pub fn read_current_l2(i2c: &I2CDriver) -> Option<u16> {
    Some((read_reg(i2c, 0x95)? / 2) as u16)
}

pub fn enable_l1(gpio: &lpc11u6x_pac::GPIO_PORT) {
    gpio.dir[2].modify(|_r, w| w.dirp2().clear_bit());
}

pub fn disable_l1(gpio: &lpc11u6x_pac::GPIO_PORT) {
    gpio.dir[2].modify(|_r, w| w.dirp2().set_bit());
}

pub fn enable_l2(gpio: &lpc11u6x_pac::GPIO_PORT) {
    gpio.dir[1].modify(|_r, w| w.dirp27().clear_bit());
}

pub fn disable_l2(gpio: &lpc11u6x_pac::GPIO_PORT) {
    gpio.dir[1].modify(|_r, w| w.dirp27().set_bit());
}

pub fn read_vid_l1(i2c: &I2CDriver) -> Option<u8> {
    read_reg(i2c, 0x7A)
}
pub fn set_voltage_l1(i2c: &I2CDriver, voltage_mv: u16) {
    let voltage_encoded: u8 = (((voltage_mv) as f32) * 0.08) as u8;
    write_reg(i2c, 0x7A, voltage_encoded);
}
pub fn set_voltage_l1_raw(i2c: &I2CDriver, voltage: u8) {
    write_reg(i2c, 0x7A, voltage);
}
pub fn set_voltage_l2(i2c: &I2CDriver, voltage_mv: u16) {
    let voltage_encoded: u8 = (((voltage_mv) as f32) * 0.08) as u8;
    write_reg(i2c, 0x7C, voltage_encoded);
}
pub fn set_voltage_l2_raw(i2c: &I2CDriver, voltage: u8) {
    write_reg(i2c, 0x7C, voltage);
}
pub fn set_offset_l2(i2c: &I2CDriver, voltage_mv: i16) {
    let voltage_encoded: u8 = (voltage_mv as f32 * 0.16) as u8;
    write_reg(i2c, 0x6F, voltage_encoded);
}

pub fn enable_dvid(i2c: &I2CDriver) {
    write_reg(i2c, 0x78, 0x86);
}
/*pub fn stupid_test(i2c: &I2CDriver) {
    let mut buffer1 = [0x23u8 << 1, 3, 0];
    let mut buffer2 = [0x23u8 << 1, 1, 40];
    let mut param = I2cParam {
        num_bytes_rec: 0,
        num_bytes_send: 3,
        buffer_ptr_rec: 0 as *mut u8,
        buffer_ptr_send: buffer1.as_mut_ptr(),
        func_pt: None,
        stop_flag: 1,
        dummy: [0, 0, 0],
    };
    let result = &mut I2cResult {
        n_bytes_recd: 0,
        n_bytes_sent: 0,
    };
    let mut err = i2c.master_transmit_poll(&mut param, result);
    if err != ErrorCode::LPC_OK {
        // TODO handle Error here
    }
    param.buffer_ptr_send = buffer2.as_mut_ptr();
    err = i2c.master_transmit_poll(&mut param, result);
    if err != ErrorCode::LPC_OK {
        // TODO handle Error here
    }
}*/

#[inline(always)]
fn bit_set(data: u8, bit: u8) -> bool {
    ((1 << bit) & data) != 0
}
pub mod faults {
    use super::*;
    pub fn input_under_voltage(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC3)?, 3))
    }
    pub fn over_temp_l1(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC3)?, 2))
    }
    pub fn over_temp_l2(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC6)?, 2))
    }
    pub fn over_current_l1(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC3)?, 4))
    }
    pub fn over_current_l2(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC6)?, 4))
    }
    pub fn over_voltage_l1(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC3)?, 5))
    }
    pub fn over_voltage_l2(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC6)?, 5))
    }
    pub fn output_off_l1(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC3)?, 6))
    }
    pub fn output_off_l2(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC6)?, 6))
    }
    pub fn power_good_error_l1(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC2)?, 3))
    }
    pub fn power_good_error_l2(i2c: &I2CDriver) -> Option<bool> {
        Some(bit_set(read_reg(i2c, 0xC5)?, 3))
    }
}
