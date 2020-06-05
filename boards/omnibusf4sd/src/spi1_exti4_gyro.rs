use core::convert::Infallible;
use core::mem::MaybeUninit;

use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::gpio::gpioa::{PA4, PA5, PA6, PA7};
use stm32f4xx_hal::gpio::gpioc::PC4;

use stm32f4xx_hal::gpio::ExtiPin;

use stm32f4xx_hal::gpio::{Alternate, AF5};
use stm32f4xx_hal::gpio::{Input, Output, PullUp, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

use rs_flight::datastructures::event::{event_nop_handler, EventHandler};
use rs_flight::drivers::mpu6000::{init as mpu6000_init, ACCELEROMETER_SENSITIVE, GYRO_SENSITIVE};
use rs_flight::hal::{sensors, AccelGyroHandler};

use mpu6000::bus::{self, DelayNs, SpiBus};
use mpu6000::measurement::{Measurement, Temperature};
use mpu6000::registers::Register;
use mpu6000::SPI_MODE;

type Spi1Pins = (PA5<Alternate<AF5>>, PA6<Alternate<AF5>>, PA7<Alternate<AF5>>);

pub struct TickDelay(u32);

impl DelayNs<u8> for TickDelay {
    fn delay_ns(&mut self, ns: u8) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

impl DelayNs<u16> for TickDelay {
    fn delay_ns(&mut self, ns: u16) {
        cortex_m::asm::delay(ns as u32 * (self.0 / 1000_000) / 1000 + 1)
    }
}

type SpiError = bus::SpiError<Error, Error, Infallible>;
static mut CS: MaybeUninit<PA4<Output<PushPull>>> = MaybeUninit::uninit();
static mut INT: MaybeUninit<PC4<Input<PullUp>>> = MaybeUninit::uninit();
static mut DMA_BUFFER: [i16; 8] = [0i16; 8];

static mut ACCEL_GYRO_HANDLER: AccelGyroHandler = event_nop_handler;
static mut TEMPERATURE_HANDLER: EventHandler<sensors::Temperature<i16>> = event_nop_handler;

#[interrupt]
unsafe fn EXTI4() {
    { &mut *INT.as_mut_ptr() }.clear_interrupt_pending_bit();
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);

    { &mut *CS.as_mut_ptr() }.set_low().ok();

    let spi1 = &(*stm32::SPI1::ptr());
    let data_register = &spi1.dr as *const _ as u32;
    let dma2 = &*(stm32::DMA2::ptr());

    // dma2 channel 3 stream 0 rx
    let stream = &dma2.st[0];
    stream.ndtr.write(|w| w.ndt().bits((DMA_BUFFER.len() as u16) * 2));
    stream.par.write(|w| w.pa().bits(data_register));
    let m0ar = &stream.m0ar;
    m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
    stream.cr.write(|w| {
        w.chsel()
            .bits(3)
            .minc()
            .incremented()
            .msize()
            .bits16()
            .dir()
            .peripheral_to_memory()
            .tcie()
            .enabled()
            .en()
            .enabled()
    });

    static READ_REG: [u8; 1] = [Register::AccelerometerXHigh as u8 | 0x80];

    // dma2 channel 3 stream 3 tx
    let stream = &dma2.st[3];
    stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16 * 2));
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(READ_REG.as_ptr() as u32));
    let cr = &stream.cr;
    cr.write(|w| w.chsel().bits(3).dir().memory_to_peripheral().en().enabled());
}

#[interrupt]
unsafe fn DMA2_STREAM0() {
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
    let dma2 = &*stm32::DMA2::ptr();
    dma2.lifcr.write(|w| w.bits(0x3D << 22 | 0x3D));

    let buf = &DMA_BUFFER;
    let acceleration = Measurement::from_array(&buf[1..], ACCELEROMETER_SENSITIVE).unwrap();
    let temperature = Temperature(buf[4]);
    let gyro = Measurement::from_array(&buf[5..], GYRO_SENSITIVE).unwrap();
    ACCEL_GYRO_HANDLER((acceleration.into(), gyro.into()));
    TEMPERATURE_HANDLER(temperature.centi_celcius());

    { &mut *CS.as_mut_ptr() }.set_high().ok();
}

pub fn init(
    spi1: stm32::SPI1,
    pins: Spi1Pins,
    mut cs: PA4<Output<PushPull>>,
    int: PC4<Input<PullUp>>,
    clocks: Clocks,
    event_handlers: (AccelGyroHandler, EventHandler<sensors::Temperature<i16>>),
    delay: &mut Delay,
    sample_rate: u16,
) -> Result<(), SpiError> {
    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, pins, SPI_MODE, freq, clocks);
    let bus = SpiBus::new(spi1, &mut cs, TickDelay(clocks.sysclk().0));
    if !mpu6000_init(bus, sample_rate, delay)? {
        return Ok(());
    }

    let freq: stm32f4xx_hal::time::Hertz = 20.mhz().into();
    let br = match clocks.pclk2().0 / freq.0 {
        0 => unreachable!(),
        1..=2 => 0b000,
        3..=5 => 0b001,
        6..=11 => 0b010,
        12..=23 => 0b011,
        24..=47 => 0b100,
        48..=95 => 0b101,
        96..=191 => 0b110,
        _ => 0b011,
    };
    let spi1 = unsafe { &(*stm32::SPI1::ptr()) };
    spi1.cr1.modify(|_, w| w.br().bits(br));
    spi1.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());

    let (accel_gyro_handler, temperature_handler) = event_handlers;
    unsafe {
        CS = MaybeUninit::new(cs);
        INT = MaybeUninit::new(int);
        ACCEL_GYRO_HANDLER = accel_gyro_handler;
        TEMPERATURE_HANDLER = temperature_handler;
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM0) }
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI4) }
    Ok(())
}
