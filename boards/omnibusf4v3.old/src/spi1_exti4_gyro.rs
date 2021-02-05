use core::convert::Infallible;
use core::mem::MaybeUninit;

use mpu6000::bus::{self, DelayNs, SpiBus};
use mpu6000::registers::Register;
use mpu6000::SPI_MODE;
use pro_flight::config;
use pro_flight::drivers::mpu6000::{init as mpu6000_init, on_dma_receive};
use stm32f4xx_hal::gpio::gpioa;
use stm32f4xx_hal::gpio::gpioc;
use stm32f4xx_hal::gpio::ExtiPin;
use stm32f4xx_hal::gpio::{Floating, Input, Output, PullUp, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

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

const DMA_SIZE: usize = 16;

static mut CS: MaybeUninit<gpioa::PA4<Output<PushPull>>> = MaybeUninit::uninit();
static mut INT: MaybeUninit<gpioc::PC4<Input<PullUp>>> = MaybeUninit::uninit();
static mut DMA_BUFFER: [u8; DMA_SIZE + 4] = [0u8; DMA_SIZE + 4]; // a little bit larger to avoid out-of-range

#[interrupt]
unsafe fn EXTI4() {
    cortex_m::interrupt::free(|_| {
        { &mut *INT.as_mut_ptr() }.clear_interrupt_pending_bit();
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    });

    { &mut *CS.as_mut_ptr() }.set_low().ok();

    let dma2 = &*(stm32::DMA2::ptr());

    let stream = &dma2.st[0];
    stream.ndtr.write(|w| w.ndt().bits(DMA_SIZE as u16));
    stream.cr.modify(|_, w| w.en().enabled());

    let stream = &dma2.st[3];
    stream.ndtr.write(|w| w.ndt().bits(DMA_SIZE as u16));
    stream.cr.modify(|_, w| w.en().enabled());
}

#[interrupt]
unsafe fn DMA2_STREAM0() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
        let dma2 = &*stm32::DMA2::ptr();
        dma2.lifcr.write(|w| w.bits(0x3D << 22 | 0x3D));
    });

    on_dma_receive(core::mem::transmute(&DMA_BUFFER), config::get().board.rotation);
    { &mut *CS.as_mut_ptr() }.set_high().ok();
}

type PA4 = gpioa::PA4<Input<Floating>>;
type PA5 = gpioa::PA5<Input<Floating>>;
type PA6 = gpioa::PA6<Input<Floating>>;
type PA7 = gpioa::PA7<Input<Floating>>;
type PC4 = gpioc::PC4<Input<PullUp>>;

fn init_dma() {
    let spi1 = unsafe { &(*stm32::SPI1::ptr()) };
    spi1.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());
    let data_register = &spi1.dr as *const _ as u32;
    let dma2 = unsafe { &*(stm32::DMA2::ptr()) };

    // dma2 channel 3 stream 0 rx
    let stream = &dma2.st[0];
    stream.par.write(|w| w.pa().bits(data_register));
    let m0ar = &stream.m0ar;
    m0ar.write(|w| w.m0a().bits(unsafe { DMA_BUFFER.as_ptr() } as u32 + 1));
    #[rustfmt::skip]
    stream.cr.write(|w| {
        w.chsel().bits(3).minc().incremented().dir().peripheral_to_memory()
            .pburst().incr16().tcie().enabled()
    });

    static READ_REG: u8 = Register::AccelerometerXHigh as u8 | 0x80;

    // dma2 channel 3 stream 3 tx
    let stream = &dma2.st[3];
    stream.par.write(|w| w.pa().bits(data_register));
    stream.m0ar.write(|w| w.m0a().bits(&READ_REG as *const _ as u32));
    stream.cr.write(|w| w.chsel().bits(3).dir().memory_to_peripheral().pburst().incr16());
}

pub fn init(
    spi1: stm32::SPI1,
    spi1_pins: (PA5, PA6, PA7),
    pa4: PA4,
    int: PC4,
    clocks: Clocks,
    sample_rate: u16,
) -> Result<bool, SpiError> {
    let mut cs = pa4.into_push_pull_output();
    let (pa5, pa6, pa7) = spi1_pins;
    let sclk = pa5.into_alternate_af5();
    let miso = pa6.into_alternate_af5();
    let mosi = pa7.into_alternate_af5();

    let freq: stm32f4xx_hal::time::Hertz = 1.mhz().into();
    let spi1 = Spi::spi1(spi1, (sclk, miso, mosi), SPI_MODE, freq, clocks);
    let bus = SpiBus::new(spi1, &mut cs, TickDelay(clocks.sysclk().0));
    if !mpu6000_init(bus, sample_rate)? {
        return Ok(false);
    }

    init_dma();

    unsafe {
        CS = MaybeUninit::new(cs);
        INT = MaybeUninit::new(int);
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM0);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM0) }
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI4);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI4) }
    Ok(true)
}
