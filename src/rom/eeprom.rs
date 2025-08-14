use super::IAP_ENTRY;
use crate::clock::get_sys_clk;

pub fn eeprom_read(addr: u32, buffer: &mut [u8], sys: &lpc11u6x_pac::SYSCON) -> super::IapStatus {
    //!
    let mut command = [0u32; 5];
    let mut result = [0u32; 4];

    command[0] = 62;
    command[1] = addr;
    command[2] = buffer.as_mut_ptr() as u32;
    command[3] = buffer.len() as u32;
    command[4] = get_sys_clk(sys) / 1000;
    let ptr = IAP_ENTRY as *const ();
    let iap_entry =
        unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn(*mut u32, *mut u32)>(ptr) };
    iap_entry_func(command.as_mut_ptr(), result.as_mut_ptr(), iap_entry);
    let status = result[0];
    status.into()
}

pub fn eeprom_write(addr: u32, pointer: u32, len: u32, sys: &lpc11u6x_pac::SYSCON) {
    let mut command = [0u32; 5];
    let mut result = [0u32; 4];

    command[0] = 61;
    command[1] = addr;
    command[2] = pointer;
    command[3] = len;
    command[4] = get_sys_clk(sys) / 1000;
    let ptr = IAP_ENTRY as *const ();
    let iap_entry =
        unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn(*mut u32, *mut u32)>(ptr) };
    iap_entry_func(command.as_mut_ptr(), result.as_mut_ptr(), iap_entry);
}

pub fn get_part_id() -> u32 {
    let mut command = [0u32; 5];
    let mut result = [0u32; 4];

    command[0] = 54;
    let ptr = IAP_ENTRY as *const ();
    let iap_entry =
        unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn(*mut u32, *mut u32)>(ptr) };
    iap_entry_func(command.as_mut_ptr(), result.as_mut_ptr(), iap_entry);
    result[0]
}

pub fn get_uid() -> u32 {
    let mut command = [0u32; 5];
    let mut result = [0u32; 5];

    command[0] = 58;
    let ptr = IAP_ENTRY as *const ();
    let iap_entry =
        unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn(*mut u32, *mut u32)>(ptr) };
    iap_entry_func(command.as_mut_ptr(), result.as_mut_ptr(), iap_entry);
    result[1]
}

pub fn black_check() {
    let mut command = [0u32; 5];
    let mut result = [0u32; 4];

    command[0] = 53;
    command[1] = 0;
    command[2] = 0;
    let ptr = IAP_ENTRY as *const ();
    let iap_entry =
        unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn(*mut u32, *mut u32)>(ptr) };
    iap_entry_func(command.as_mut_ptr(), result.as_mut_ptr(), iap_entry);
}
/*
pub fn part_id_read(addr: u32, pointer: u32, len: u32, sys: &lpc11u6x_pac::SYSCON){
    let mut command = [0u32;5];
    let result = [0u32;4];
    let ptr = IAP_ENTRY as *const ();
    let iap_entry = unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn ([u32;5], [u32;4])>(ptr) };

    command[0] = 54;
    unsafe{iap_entry(command,result)};
}
pub fn part_id_read(addr: u32, pointer: u32, len: u32, sys: &lpc11u6x_pac::SYSCON){
    let mut command = [0u32;5];
    let result = [0u32;4];
    let ptr = IAP_ENTRY as *const ();
    let iap_entry = unsafe { core::mem::transmute::<*const (), unsafe extern "C" fn ([u32;5], [u32;4])>(ptr) };

    command[0] = 54;
    let err = unsafe{iap_entry(command,result)};
    let a = 7;
}*/
fn iap_entry_func(
    command: *mut u32,
    result: *mut u32,
    func_ptr: unsafe extern "C" fn(*mut u32, *mut u32),
) {
    unsafe { (func_ptr)(command, result) };
}
