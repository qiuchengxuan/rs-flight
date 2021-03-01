use chips::stm32f4::flash::{Error, Flash, Sector};
use pro_flight::hal;

pub struct FlashWrapper(Flash);

impl FlashWrapper {
    pub fn new(flash: Flash) -> Self {
        Self(flash)
    }
}

impl hal::flash::Flash<u32> for FlashWrapper {
    type Error = Error;

    fn erase(&mut self, address: usize) -> Result<(), Error> {
        self.0.erase(Sector::from_address(address).unwrap())
    }

    fn program(&mut self, address: usize, words: &[u32]) -> Result<(), Error> {
        self.0.program(address, words)
    }
}
