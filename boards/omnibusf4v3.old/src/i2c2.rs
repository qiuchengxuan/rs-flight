use stm32f4xx_hal::gpio::gpiob;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::i2c::{Error, I2c};
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::{prelude::*, stm32};

use pro_flight::components::schedule::Schedulable;
use pro_flight::drivers::magnetometer::qmc5883l;

type PB10 = gpiob::PB10<Input<Floating>>;
type PB11 = gpiob::PB11<Input<Floating>>;

pub fn init(
    i2c2: stm32::I2C2,
    i2c2_pins: (PB10, PB11),
    clocks: Clocks,
) -> Result<Option<impl Schedulable>, Error> {
    let (pb10, pb11) = i2c2_pins;
    let scl = pb10.into_alternate_af4().set_open_drain();
    let sda = pb11.into_alternate_af4().set_open_drain();
    // TODO: scan i2c bus
    qmc5883l::init(I2c::i2c2(i2c2, (scl, sda), 400.khz(), clocks))
}
