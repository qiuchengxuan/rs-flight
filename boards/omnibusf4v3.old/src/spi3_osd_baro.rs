use core::mem::MaybeUninit;

use bmp280::bus::{DelayNs, DummyOutputPin, SpiBus};
use bmp280::registers::Register;
use crc::Hasher32;
use max7456::not_null_writer::NotNullWriter;
use max7456::SPI_MODE;
use pro_flight::components::ascii_hud::AsciiHud;
use pro_flight::components::schedule::{Rate, Schedulable};
use pro_flight::components::telemetry::TelemetryData;
use pro_flight::config;
use pro_flight::datastructures::data_source::StaticData;
use pro_flight::datastructures::Ratio;
use pro_flight::drivers::barometer::bmp280::{init as bmp280_init, on_dma_receive};
use pro_flight::drivers::max7456::init as max7456_init;
use pro_flight::drivers::shared_spi::{SharedSpi, VirtualSpi};
use stm32f4xx_hal::gpio::gpioa::PA15;
use stm32f4xx_hal::gpio::gpiob::PB3;
use stm32f4xx_hal::gpio::gpioc::{PC10, PC11, PC12};
use stm32f4xx_hal::gpio::Input;
use stm32f4xx_hal::gpio::{Floating, Output, PushPull};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::spi::{Error, Spi};
use stm32f4xx_hal::{prelude::*, stm32};

const STREAM5_OFFSET: usize = 6;
const STREAM2_OFFSET: usize = 16;

// NOTE: actually empty, no need to initialize
static mut CS_BMP280: MaybeUninit<PB3<Output<PushPull>>> = MaybeUninit::uninit();
static mut CS_MAX7456: MaybeUninit<PA15<Output<PushPull>>> = MaybeUninit::uninit();
static mut DMA_BUFFER: [u8; 8] = [0u8; 8];

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

#[interrupt]
unsafe fn DMA1_STREAM2() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
        let dma1 = &*stm32::DMA1::ptr();
        dma1.hifcr.write(|w| w.bits(0x3D << STREAM5_OFFSET));
        dma1.lifcr.write(|w| w.bits(0x3D << STREAM2_OFFSET));
    });

    on_dma_receive(&DMA_BUFFER);
}

pub struct BaroScheduler;

fn spi3_dma_ready() -> bool {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    dma1.st[2].cr.read().en().is_disabled() && dma1.st[5].cr.read().en().is_disabled()
}

fn select_bmp280() {
    unsafe { { &mut *CS_BMP280.as_mut_ptr() }.set_low().ok() };
    unsafe { { &mut *CS_MAX7456.as_mut_ptr() }.set_high().ok() };
}

fn select_max7456() {
    unsafe { { &mut *CS_BMP280.as_mut_ptr() }.set_high().ok() };
    unsafe { { &mut *CS_MAX7456.as_mut_ptr() }.set_low().ok() };
}

fn spi3_prepare_rx(dma_buffer: &[u8]) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    unsafe { dma1.lifcr.write(|w| w.bits(0x3D << STREAM2_OFFSET)) };
    let stream = &dma1.st[2]; // dma1 channel 0 stream 2 rx
    stream.m0ar.write(|w| w.m0a().bits(dma_buffer.as_ptr() as u32));
    stream.ndtr.write(|w| w.ndt().bits(dma_buffer.len() as u16));
    stream.cr.modify(|_, w| w.en().enabled());
}

fn spi3_start_tx(dma_buffer: &[u8], size: usize) {
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    unsafe { dma1.hifcr.write(|w| w.bits(0x3D << STREAM5_OFFSET)) };
    let stream = &dma1.st[5];
    stream.m0ar.write(|w| w.m0a().bits(dma_buffer.as_ptr() as u32));
    stream.ndtr.write(|w| w.ndt().bits(size as u16));
    if dma_buffer.len() != size {
        stream.cr.modify(|_, w| w.minc().fixed().en().enabled());
    } else {
        stream.cr.modify(|_, w| w.minc().incremented().en().enabled());
    }
}

impl Schedulable for BaroScheduler {
    fn rate(&self) -> Rate {
        16
    }

    fn schedule(&mut self) -> bool {
        if !spi3_dma_ready() {
            return false;
        }
        select_bmp280();

        spi3_prepare_rx(unsafe { &DMA_BUFFER });
        spi3_start_tx(&[Register::PressureMsb as u8 | 0x80], unsafe { DMA_BUFFER.len() });
        true
    }
}

pub struct OSDScheduler<T>(AsciiHud<T>);

impl<T: StaticData<TelemetryData>> Schedulable for OSDScheduler<T> {
    fn rate(&self) -> Rate {
        config::get().osd.refresh_rate as Rate
    }

    fn schedule(&mut self) -> bool {
        if !spi3_dma_ready() {
            return false;
        }
        select_max7456();

        let screen = self.0.draw();
        static mut OSD_DMA_BUFFER: [u8; 800] = [0u8; 800];
        let mut dma_buffer = unsafe { &mut OSD_DMA_BUFFER[..] };
        let mut writer = NotNullWriter::new(screen, Default::default());
        let display = writer.write(&mut dma_buffer);
        spi3_start_tx(&display.0, display.0.len());
        true
    }
}

fn init_dma() {
    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    let data_register = &spi3.dr as *const _ as u32;
    let dma1 = unsafe { &*(stm32::DMA1::ptr()) };
    let stream = &dma1.st[2]; // dma1 channel 0 stream 2 rx
    stream.par.write(|w| w.pa().bits(data_register));
    stream.cr.write(|w| {
        w.chsel().bits(0).minc().incremented().dir().peripheral_to_memory().tcie().enabled()
    });

    let stream = &dma1.st[5]; // dma1 channel 0 stream 5 tx
    stream.par.write(|w| w.pa().bits(data_register));
    stream.cr.write(|w| w.chsel().bits(0).dir().memory_to_peripheral());
}

pub fn init<'a, CRC: Hasher32>(
    spi3: stm32::SPI3,
    spi3_pins: (PC10<Input<Floating>>, PC11<Input<Floating>>, PC12<Input<Floating>>),
    chip_selects: (PA15<Input<Floating>>, PB3<Input<Floating>>),
    crc: &mut CRC,
    clocks: Clocks,
    telemetry: impl StaticData<TelemetryData>,
) -> Result<(impl Schedulable, impl Schedulable), Error> {
    let cs_osd = chip_selects.0.into_push_pull_output();
    let cs_baro = chip_selects.1.into_push_pull_output();
    let (pc10, pc11, pc12) = spi3_pins;
    let sclk = pc10.into_alternate_af6();
    let miso = pc11.into_alternate_af6();
    let mosi = pc12.into_alternate_af6();
    let freq: stm32f4xx_hal::time::Hertz = 10.mhz().into();
    let spi3 = Spi::spi3(spi3, (sclk, miso, mosi), SPI_MODE, freq, clocks);
    let spi = SharedSpi::new(spi3, (cs_baro, cs_osd));

    let mut dummy_cs = DummyOutputPin {};
    let bus = SpiBus::new(VirtualSpi::new(&spi, 0), &mut dummy_cs, TickDelay(clocks.sysclk().0));
    if !bmp280_init(bus).is_ok() {
        warn!("BMP280 init not ok")
    }
    max7456_init(VirtualSpi::new(&spi, 1), crc)?;

    let spi3 = unsafe { &(*stm32::SPI3::ptr()) };
    spi3.cr2.modify(|_, w| w.txdmaen().enabled().rxdmaen().enabled());
    init_dma();

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA1_STREAM2);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA1_STREAM2) }
    let cfg = &config::get().osd;
    let hud = AsciiHud::new(telemetry, cfg.fov, Ratio(12, 18).into(), cfg.aspect_ratio.into());
    Ok((BaroScheduler, OSDScheduler(hud)))
}
