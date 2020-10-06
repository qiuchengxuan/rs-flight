use alloc::boxed::Box;

use rs_flight::config::SerialConfig;
use rs_flight::drivers::gnss::GNSS;
use rs_flight::drivers::uart::Device;
use stm32f4xx_hal::gpio::gpioa;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::stm32;

use crate::stm32f4::{alloc_by_config, to_serial_config};

type PA9 = gpioa::PA9<Input<Floating>>;
type PA10 = gpioa::PA10<Input<Floating>>;

const HTIF_OFFSET: usize = 4;
const STREAM5_OFFSET: usize = 6;

static mut DEVICE: Option<Device> = None;

#[interrupt]
unsafe fn DMA2_STREAM5() {
    let mut half = false;
    let mut buffer: &[u8] = &[0u8; 0];
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM5);
        let dma2 = &*stm32::DMA2::ptr();
        half = dma2.hisr.read().bits() & (1 << (HTIF_OFFSET + STREAM5_OFFSET)) > 0;
        dma2.hifcr.write(|w| w.bits(0x3D << STREAM5_OFFSET));
        let address = dma2.st[5].m0ar.read().bits();
        let size = *((address - 2) as *const u16) as usize;
        buffer = core::slice::from_raw_parts(address as *const _, size);
    });
    if let Some(ref mut device) = DEVICE {
        device.handle(buffer, half);
    }
}

pub fn init(
    usart1: stm32::USART1,
    pins: (PA9, PA10),
    config: &SerialConfig,
    clocks: Clocks,
) -> Option<&'static mut Device> {
    let (pa9, pa10) = pins;
    let pins = (pa9.into_alternate_af7(), pa10.into_alternate_af7());
    Serial::usart1(usart1, pins, to_serial_config(&config), clocks).unwrap();

    unsafe {
        let usart = &*stm32::USART1::ptr();

        let dma_buffer = Box::leak(alloc_by_config(&config));
        let address = dma_buffer.as_ptr() as usize + 2;
        let size = dma_buffer.len() - 2;
        *(dma_buffer as *mut _ as *mut u16) = size as u16;
        debug!("Alloc DMA buffer at {:#X} size {} on USART1", address, size);

        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[5];
        stream.ndtr.write(|w| w.ndt().bits(size as u16));
        stream.par.write(|w| w.pa().bits(&usart.dr as *const _ as u32));
        stream.m0ar.write(|w| w.m0a().bits(address as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(4).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .pl().very_high().htie().enabled().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM5);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM5);
    }

    let device = match config {
        SerialConfig::GNSS(gnss) => Device::GNSS(GNSS::new(gnss.protocol)),
        _ => return None,
    };
    unsafe { DEVICE = Some(device) }
    unsafe { DEVICE.as_mut() }
}
