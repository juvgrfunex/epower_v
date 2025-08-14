use crate::i2c::{I2CDriver, I2cParam, I2cResult};
use crate::{clock, rom::ErrorCode, timer};

fn flip_character(mut char: u8) -> u8 {
    char = swap_bits(char, 0, 3);
    char = swap_bits(char, 2, 5);
    char = swap_bits(char, 1, 4);
    char
}

fn swap_bits(mut data: u8, bit_index1: u8, bit_index2: u8) -> u8 {
    let val_bit1 = data & (1 << bit_index1);
    let val_bit2 = data & (1 << bit_index2);

    data &=  !(1 << bit_index2);
    if val_bit1 != 0 {
        data ^= 1 << bit_index2;
    }

    data &= !(1 << bit_index1);
    if val_bit2 != 0 {
        data ^= 1 << bit_index1;
    }
    data
}
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Character {
    Off,
    Dot,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl From<Character> for u8 {
    fn from(char: Character) -> Self {
        match char {
            Character::Off => 0,
            Character::Dot => 128,
            Character::Zero => 63,
            Character::One => 6,
            Character::Two => 91,
            Character::Three => 79,
            Character::Four => 102,
            Character::Five => 109,
            Character::Six => 125,
            Character::Seven => 7,
            Character::Eight => 127,
            Character::Nine => 111,
            Character::A => 119,
            Character::B => 124,
            Character::C => 57,
            Character::D => 94,
            Character::E => 121,
            Character::F => 113,
        }
    }
}

pub const fn digit_into_character(digit: u8) -> Character {
    match digit {
        0 => Character::Zero,
        1 => Character::One,
        2 => Character::Two,
        3 => Character::Three,
        4 => Character::Four,
        5 => Character::Five,
        6 => Character::Six,
        7 => Character::Seven,
        8 => Character::Eight,
        9 => Character::Nine,
        10 => Character::A,
        11 => Character::B,
        12 => Character::C,
        13 => Character::D,
        14 => Character::E,
        15 => Character::F,
        _ => Character::Off,
    }
}

pub const fn current_into_row(data: u16) -> [Character; 4] {
    let d3 = digit_into_character((data / 100) as u8);
    let d2 = digit_into_character(((data % 100) / 10) as u8);
    let d1 = digit_into_character((data % 10) as u8);
    let d0 = Character::A;
    [d3, d2, d1, d0]
}

pub const fn temp_into_row(data: u8) -> [Character; 4] {
    let d3 = digit_into_character(data / 100);
    let d2 = digit_into_character((data % 100) / 10);
    let d1 = digit_into_character(data % 10);
    let d0 = Character::C;
    [d3, d2, d1, d0]
}

pub const fn voltage_into_row(data: f32) -> [Character; 4] {
    let mv = data as i32;
    if mv >= 0 && mv < 10000 {
        let d3 = digit_into_character((mv / 1000) as u8);
        let d2 = digit_into_character(((mv % 1000) / 100) as u8);
        let d1 = digit_into_character(((mv % 100) / 10) as u8);
        let d0 = digit_into_character((mv % 10) as u8);
        [d3, d2, d1, d0]
    } else {
        [Character::F; 4]
    }
}

#[derive(PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Normal,
    FLipped,
}

impl From<u8> for Orientation {
    fn from(value: u8) -> Self {
        if value == 0 {
            Orientation::Normal
        } else {
            Orientation::FLipped
        }
    }
}

impl From<Orientation> for u8 {
    fn from(val: Orientation) -> u8 {
        match val {
            Orientation::Normal => 0,
            Orientation::FLipped => 1,
        }
    }
}

pub struct Display {
    orientation: Orientation,
    digit: u8,
    data: [u8; 8],
    timer: timer::Timer<lpc11u6x_pac::CT16B0>,
}

impl Display {
    pub fn new(
        gpio: &lpc11u6x_pac::GPIO_PORT,
        sys: &lpc11u6x_pac::SYSCON,
        i2c: &I2CDriver,
        timer: lpc11u6x_pac::CT16B0,
    ) -> Self {
        let ct160 = timer::enable_timer(sys, timer::TimerSel::CT16B0, timer);
        let freq = clock::get_sys_clk(sys);
        ct160.reset();
        ct160.int_on_match(1);
        ct160.set_prescale(0);
        ct160.set_match(1, ((freq / 1000) - 1) as u16);
        ct160.reset_on_match(1);

        let mut buffer = [0x21u8 << 1, 3, 0];

        let mut param = I2cParam {
            num_bytes_rec: 0,
            num_bytes_send: 3,
            buffer_ptr_rec: core::ptr::null_mut::<u8>(),
            buffer_ptr_send: buffer.as_mut_ptr(),
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
        buffer = [0x21u8 << 1, 1, 0];
        param.buffer_ptr_send = buffer.as_mut_ptr();
        err = i2c.master_transmit_poll(&mut param, result);
        if err != ErrorCode::LPC_OK {
            // TODO handle Error here
        }
        let d = Display {
            orientation: Orientation::Normal,
            digit: 0,
            data: [0, 0, 0, 0, 0, 0, 0, 0],
            timer: ct160,
        };
        d.write_i2c(i2c);
        d.update_out(gpio);
        d.timer.start();
        d
    }

    pub fn set_display_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    pub fn set_char<T: Into<u8>>(&mut self, index: usize, character: T) {
        if self.orientation == Orientation::FLipped {
            self.data[index] = flip_character(character.into());
        } else {
            self.data[index] = character.into();
        }
    }
    pub fn set_all<T: Into<u8> + Copy>(&mut self, data: [T; 8]) {
        match self.orientation {
            Orientation::FLipped => {
                for (i, character) in self.data.iter_mut().rev().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
            Orientation::Normal => {
                for (i, character) in self.data.iter_mut().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
        };
    }

    pub fn set_row_top<T: Into<u8> + Copy>(&mut self, data: [T; 4]) {
        match self.orientation {
            Orientation::FLipped => {
                for (i, character) in self.data[0..4].iter_mut().rev().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
            Orientation::Normal => {
                for (i, character) in self.data[0..4].iter_mut().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
        };
    }

    pub fn set_row_bottom<T: Into<u8> + Copy>(&mut self, data: [T; 4]) {
        match self.orientation {
            Orientation::FLipped => {
                for (i, character) in self.data[4..8].iter_mut().rev().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
            Orientation::Normal => {
                for (i, character) in self.data[4..8].iter_mut().enumerate() {
                    if self.orientation == Orientation::FLipped {
                        *character = flip_character(data[i].into());
                    } else {
                        *character = data[i].into();
                    }
                }
            }
        }
    }
    fn write_i2c(&self, i2c: &I2CDriver) {
        let mut buffer = [0x21u8 << 1, 1, self.data[self.digit as usize]];

        let mut param = I2cParam {
            num_bytes_rec: 0,
            num_bytes_send: 3,
            buffer_ptr_rec: core::ptr::null_mut::<u8>(),
            buffer_ptr_send: buffer.as_mut_ptr(),
            func_pt: None,
            stop_flag: 1,
            dummy: [0, 0, 0],
        };
        let result = &mut I2cResult {
            n_bytes_recd: 0,
            n_bytes_sent: 0,
        };
        let _err = i2c.master_transmit_poll(&mut param, result);
    }
    pub fn increment_digit(&mut self, gpio: &lpc11u6x_pac::GPIO_PORT, i2c: &I2CDriver) {
        self.timer.clear_int();
        self.clear_out(gpio);
        if self.digit == 7 {
            self.digit = 0;
        } else {
            self.digit += 1;
        }
        self.write_i2c(i2c);
        self.update_out(gpio);
    }

    fn update_out(&self, gpio: &lpc11u6x_pac::GPIO_PORT) {
        match self.digit {
            0 => gpio.dir[1].modify(|_r, w| w.dirp7().set_bit()),
            1 => gpio.dir[2].modify(|_r, w| w.dirp5().set_bit()),
            2 => gpio.dir[1].modify(|_r, w| w.dirp26().set_bit()),
            3 => gpio.dir[0].modify(|_r, w| w.dirp21().set_bit()),
            4 => gpio.dir[1].modify(|_r, w| w.dirp19().set_bit()),
            5 => gpio.dir[0].modify(|_r, w| w.dirp2().set_bit()),
            6 => gpio.dir[1].modify(|_r, w| w.dirp20().set_bit()),
            7 => gpio.dir[2].modify(|_r, w| w.dirp6().set_bit()),
            _ => (),
        }
    }
    fn clear_out(&self, gpio: &lpc11u6x_pac::GPIO_PORT) {
        gpio.dir[2].modify(|_r, w| w.dirp6().clear_bit());
        gpio.dir[1].modify(|_r, w| w.dirp20().clear_bit());
        gpio.dir[1].modify(|_r, w| w.dirp26().clear_bit());
        gpio.dir[1].modify(|_r, w| w.dirp7().clear_bit());
        gpio.dir[0].modify(|_r, w| w.dirp21().clear_bit());
        gpio.dir[0].modify(|_r, w| w.dirp2().clear_bit());
        gpio.dir[1].modify(|_r, w| w.dirp19().clear_bit());
        gpio.dir[2].modify(|_r, w| w.dirp5().clear_bit());
    }
}
