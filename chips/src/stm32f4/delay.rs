use embedded_hal::blocking::delay::DelayUs;

use super::clock::HCLK;

pub struct TickDelay;

impl<T: Into<u32>> DelayUs<T> for TickDelay {
    fn delay_us(&mut self, us: T) {
        cortex_m::asm::delay(us.into() * (HCLK / 1000) / 1000 + 1)
    }
}
