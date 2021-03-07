use core::fmt::Debug;

use chips::stm32f4::delay::TickDelay;
use drone_cortexm::thr::ThrNvic;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use mpu6000::bus::{SpiBus, SpiError};
use mpu6000::MPU6000;
pub use mpu6000::SPI_MODE;
use pro_flight::drivers::mpu6000::MPU6000Init;
use pro_flight::sys::timer::SysTimer;

const SAMPLE_RATE: usize = 1000;

pub fn init<E: Debug, PE: Debug, SPI, CS>(spi1: SPI, cs: CS, _thread: impl ThrNvic)
where
    SPI: Transfer<u8, Error = E> + Write<u8, Error = E>,
    CS: OutputPin<Error = PE>,
{
    let mut mpu6000 = MPU6000::new(SpiBus::new(spi1, cs, TickDelay {}));
    let mut delay = SysTimer::new();
    let result: Result<(), SpiError<E, E, PE>> = (|| {
        mpu6000.reset(&mut delay)?;
        if !mpu6000.verify()? {
            error!("MPU6000 not detected");
            return Ok(());
        }
        let _convertor = mpu6000.init(SAMPLE_RATE as u16)?;
        Ok(())
    })();
    if let Some(err) = result.err() {
        error!("MPU6000 init failed: {:?}", err);
    }
}
