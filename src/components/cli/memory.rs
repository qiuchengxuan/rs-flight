use core::time::Duration;

use embedded_hal::timer::CountDown;

use crate::sys::timer::SysTimer;

extern "Rust" {
    fn memory_valid_address(address: usize) -> bool;
}

#[macro_export]
macro_rules! no_memory_valid_address {
    () => {
        #[no_mangle]
        fn memory_valid_address(address: usize) -> bool {
            false
        }
    };
}

pub fn dump(line: &str) {
    let mut iter = line.split(' ');
    let mut address: usize = 0;
    if let Some(word) = iter.next() {
        if let Some(addr) = usize::from_str_radix(word, 16).ok() {
            address = addr;
        }
    }
    if address == 0 {
        return;
    }
    if !unsafe { memory_valid_address(address) } {
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

fn _read(line: &str) -> Option<usize> {
    let mut split = line.split(' ');
    if let Some(address) = split.next().map(|s| usize::from_str_radix(s, 16).ok()).flatten() {
        if unsafe { memory_valid_address(address) } {
            return Some(unsafe { *(address as *const usize) });
        }
    }
    None
}

pub fn read(line: &str) {
    if let Some(value) = _read(line) {
        println!("Result: {}", value)
    }
}

pub fn readx(line: &str) {
    if let Some(value) = _read(line) {
        println!("Result: {:x}", value);
    }
}

pub fn writex(line: &str) {
    let mut iter = line.split(' ').flat_map(|w| usize::from_str_radix(w, 16).ok());
    if let Some(address) = iter.next() {
        if let Some(value) = iter.next() {
            if unsafe { memory_valid_address(address) } {
                unsafe { *(address as *mut usize) = value };
                let mut count_down = SysTimer::new();
                count_down.start(Duration::from_millis(1));
                nb::block!(count_down.wait()).ok();
                let value = unsafe { *(address as *const usize) };
                println!("Write result: {:x?}", value);
            }
        }
    }
}
