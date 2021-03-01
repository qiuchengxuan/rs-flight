use core::time::Duration;

use embedded_hal::timer::CountDown;

use crate::sys::timer::SysTimer;

pub type MemoryAddressValidator = fn(u32) -> bool;

fn no_address_validator(_: u32) -> bool {
    true
}

static mut MEMORY_ADDRESS_VALIDATOR: MemoryAddressValidator = no_address_validator;

pub fn init(validator: MemoryAddressValidator) {
    unsafe { MEMORY_ADDRESS_VALIDATOR = validator };
}

pub fn dump(line: &str) {
    let mut iter = line.split(' ');
    let mut address: u32 = 0;
    if let Some(word) = iter.next() {
        if let Some(addr) = u32::from_str_radix(word, 16).ok() {
            address = addr;
        }
    }
    if address == 0 {
        return;
    }
    if !unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
        return;
    }
    let mut size: usize = 0;
    if let Some(word) = iter.next() {
        if let Some(sz) = word.parse().ok() {
            size = sz
        }
    }
    let slice = unsafe { core::slice::from_raw_parts(address as *const u8, size) };
    println!("Result: {:x?}", slice)
}

fn _read(line: &str, hex: bool) {
    let mut split = line.split(' ');
    if let Some(address) = split.next().map(|s| u32::from_str_radix(s, 16).ok()).flatten() {
        if unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
            return if hex {
                let value = unsafe { *(address as *const u32) };
                println!("Result: {:x}", value)
            } else {
                let value = unsafe { *(address as *const u32) };
                println!("Result: {}", value)
            };
        }
    }
}

pub fn read(line: &str) {
    _read(line, false)
}

pub fn readx(line: &str) {
    _read(line, true)
}

pub fn writex(line: &str) {
    let mut iter = line.split(' ').flat_map(|w| u32::from_str_radix(w, 16).ok());
    if let Some(address) = iter.next() {
        if let Some(value) = iter.next() {
            if unsafe { MEMORY_ADDRESS_VALIDATOR }(address) {
                unsafe { *(address as *mut u32) = value };
                let mut count_down = SysTimer::new();
                count_down.start(Duration::from_millis(1));
                nb::block!(count_down.wait()).ok();
                let value = unsafe { *(address as *const u32) };
                println!("Write result: {:x?}", value);
            }
        }
    }
}
