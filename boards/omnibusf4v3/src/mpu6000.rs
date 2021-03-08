use embedded_hal::blocking::spi::Transfer;
use embedded_hal::digital::v2::OutputPin;

const SAMPLE_RATE: usize = 1000;

pub fn init<E>(spi1: impl Transfer<u8, Error = E>, cs: impl OutputPin) {}
