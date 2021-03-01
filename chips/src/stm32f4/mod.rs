pub mod clock;
pub mod crc;
pub mod dfu;
pub mod flash;
pub mod rtc;
pub mod systick;
pub mod usb_serial;

pub fn valid_memory_address(address: u32) -> bool {
    match address {
        0xE000_E008..=0xE000_EF44 => true,
        0x4000_0000..=0xA000_0FFF => true,
        0x2000_0000..=0x2001_FFFF => true,
        0x1000_0000..=0x1000_FFFF => true,
        0x0800_0000..=0x080E_0000 => true,
        _ => false,
    }
}
