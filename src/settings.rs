use crate::{
    disp::{self, Orientation},
    FIRMWARE_VERSION,
};

const DISPLAY_TOP_MODE_OFFSET: usize = 0;
const DISPLAY_BOTTOM_MODE_OFFSET: usize = 1;
const L1_BOOT_VOLTAGE_OFFSET: usize = 2;
const L2_BOOT_VOLTAGE_OFFSET: usize = 3;
const L1_ENABLED_OFFSET: usize = 4;
const L2_ENABLED_OFFSET: usize = 5;
const DISPLAY_ORIENTATION_OFFSET: usize = 6;

const SETTINGS_VERSION_OFFSET: usize = 59;

const VALID_SETTING_OFFSETS: [usize; 7] = [
    DISPLAY_TOP_MODE_OFFSET,
    DISPLAY_BOTTOM_MODE_OFFSET,
    L1_BOOT_VOLTAGE_OFFSET,
    L2_BOOT_VOLTAGE_OFFSET,
    L1_ENABLED_OFFSET,
    L2_ENABLED_OFFSET,
    DISPLAY_ORIENTATION_OFFSET,
];
const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);

pub struct Settings {
    sys: lpc11u6x_pac::SYSCON,
    cache: [u8; 60],
}

fn default_settings() -> [u8; 64] {
    let mut settings = [0; 64];
    settings[DISPLAY_TOP_MODE_OFFSET] = 0;
    settings[DISPLAY_BOTTOM_MODE_OFFSET] = 1;
    settings[L1_BOOT_VOLTAGE_OFFSET] = 0x58; // 1100mv
    settings[L2_BOOT_VOLTAGE_OFFSET] = 0x6C; // 135xmv
    settings[L1_ENABLED_OFFSET] = 1;
    settings[L2_ENABLED_OFFSET] = 0;
    settings[SETTINGS_VERSION_OFFSET] = FIRMWARE_VERSION;
    settings[DISPLAY_ORIENTATION_OFFSET] = Orientation::default().into();
    let checksum = CRC.checksum(&settings[0..60]);
    settings[60..64].copy_from_slice(&checksum.to_le_bytes());
    settings
}

fn read_buffer_from_eeprom(sys: &lpc11u6x_pac::SYSCON) -> ([u8; 60], [u8; 4]) {
    //! First buffer is the 60 data bytes, the last of which contains the version
    //!
    //! Second buffer is the 4 cheksum bytes
    let mut buffer = [0; 64];
    crate::rom::eeprom::eeprom_read(0x400, &mut buffer, sys);
    let (data_buffer, checksum_buffer) = buffer.split_at(60);
    (
        data_buffer.try_into().expect("data_buffer is 60 bytes"),
        checksum_buffer
            .try_into()
            .expect("checksum_buffer is 4 bytes"),
    )
}

impl Settings {
    pub fn new(sys: lpc11u6x_pac::SYSCON) -> Self {
        let (data_buffer, checksum_buffer) = read_buffer_from_eeprom(&sys);

        let checksum = CRC.checksum(&data_buffer);
        if checksum == u32::from_le_bytes(checksum_buffer)
            && data_buffer[SETTINGS_VERSION_OFFSET] == crate::FIRMWARE_VERSION
        {
            Settings {
                sys,
                cache: data_buffer,
            }
        } else {
            let settings = default_settings();
            crate::rom::eeprom::eeprom_write(
                0x400,
                settings.as_ptr() as u32,
                settings.len() as u32,
                &sys,
            );
            let mut cache = [0; 60];
            cache.copy_from_slice(&settings[0..60]);
            Settings { sys, cache }
        }
    }

    pub fn write_cache_to_eeprom(&mut self) {
        crate::rom::eeprom::eeprom_write(
            0x400,
            self.cache.as_mut_ptr() as u32,
            self.cache.len() as u32,
            &self.sys,
        );
        let checksum = CRC.checksum(&self.cache);
        let mut cheksum_bytes = checksum.to_le_bytes();
        crate::rom::eeprom::eeprom_write(
            0x400 + self.cache.len() as u32,
            cheksum_bytes.as_mut_ptr() as u32,
            4,
            &self.sys,
        );
    }

    pub fn get_setting_value(&self, setting_id: u8) -> u8 {
        let setting_id = setting_id as usize;
        if VALID_SETTING_OFFSETS.contains(&setting_id) {
            self.cache[setting_id]
        } else {
            0
        }
    }

    pub fn set_setting_value(
        &mut self,
        setting_id: u8,
        value: u8,
        display: &mut disp::Display,
        gpio: &lpc11u6x_pac::GPIO_PORT,
    ) {
        let setting_id = setting_id as usize;
        if VALID_SETTING_OFFSETS.contains(&setting_id) {
            self.write_setting(setting_id, value);
            match setting_id {
                DISPLAY_TOP_MODE_OFFSET => {}
                DISPLAY_BOTTOM_MODE_OFFSET => {}
                L1_BOOT_VOLTAGE_OFFSET => {}
                L2_BOOT_VOLTAGE_OFFSET => {}
                L1_ENABLED_OFFSET => {
                    if value == 1 {
                        crate::vrm::enable_l1(gpio);
                    } else {
                        crate::vrm::disable_l1(gpio);
                    }
                }
                L2_ENABLED_OFFSET => {
                    if value == 1 {
                        crate::vrm::enable_l2(gpio);
                    } else {
                        crate::vrm::disable_l2(gpio);
                    }
                }
                DISPLAY_ORIENTATION_OFFSET => {
                    display.set_display_orientation(value.into());
                }
                _ => {}
            }
        }
    }
}

impl Settings {
    fn write_setting(&mut self, setting_id: usize, value: u8) {
        if self.cache[setting_id] != value {
            self.cache[setting_id] = value;
            self.write_cache_to_eeprom();
        }
    }

    pub fn get_disp_top_mode(&self) -> u8 {
        self.cache[DISPLAY_TOP_MODE_OFFSET]
    }
    pub fn get_disp_bottom_mode(&self) -> u8 {
        self.cache[DISPLAY_BOTTOM_MODE_OFFSET]
    }

    pub fn is_l1_enabled(&self) -> bool {
        self.cache[L1_ENABLED_OFFSET] == 1
    }
    pub fn enable_l1(&mut self) {
        self.write_setting(L1_ENABLED_OFFSET, 1);
    }
    pub fn disable_l1(&mut self) {
        self.write_setting(L1_ENABLED_OFFSET, 0);
    }
    pub fn is_l2_enabled(&self) -> bool {
        self.cache[L2_ENABLED_OFFSET] == 1
    }
    pub fn enable_l2(&mut self) {
        self.write_setting(L2_ENABLED_OFFSET, 1);
    }
    pub fn disable_l2(&mut self) {
        self.write_setting(L2_ENABLED_OFFSET, 0);
    }

    pub fn l1_boot_voltage(&self) -> u8 {
        self.cache[L1_BOOT_VOLTAGE_OFFSET]
    }

    pub fn l2_boot_voltage(&self) -> u8 {
        self.cache[L2_BOOT_VOLTAGE_OFFSET]
    }
    pub fn get_display_orientation(&self) -> Orientation {
        self.cache[DISPLAY_ORIENTATION_OFFSET].into()
    }
    pub fn set_display_orientation(&mut self, orient: Orientation) {
        self.write_setting(DISPLAY_ORIENTATION_OFFSET, orient.into());
    }
}
