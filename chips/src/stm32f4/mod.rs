pub mod clock;
pub mod crc;
pub mod delay;
pub mod dfu;
pub mod flash;
pub mod rtc;
pub mod spi;
pub mod systick;
pub mod usb_serial;

#[no_mangle]
pub fn memory_valid_address(address: usize) -> bool {
    match address {
        0xE000_E008..=0xE000_EF44 => true,
        0x4000_0000..=0xA000_0FFF => true,
        0x2000_0000..=0x2001_FFFF => true,
        0x1000_0000..=0x1000_FFFF => true,
        0x0800_0000..=0x080E_0000 => true,
        _ => false,
    }
}
