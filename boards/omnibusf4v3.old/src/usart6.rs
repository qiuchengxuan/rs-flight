use alloc::boxed::Box;

use pro_flight::config::SerialConfig;
use pro_flight::drivers::sbus::SbusReceiver;
use pro_flight::drivers::uart::Device;
use stm32f4xx_hal::gpio::gpioc;
use stm32f4xx_hal::gpio::{Floating, Input};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::Serial;
use stm32f4xx_hal::stm32;

use crate::stm32f4::{alloc_by_config, to_serial_config};

type PC6 = gpioc::PC6<Input<Floating>>;
type PC7 = gpioc::PC7<Input<Floating>>;

const HTIF_OFFSET: usize = 4;
const STREAM1_OFFSET: usize = 6;

static mut DEVICE: Option<Device> = None;

#[interrupt]
unsafe fn DMA2_STREAM1() {
    let mut half = false;
    let mut buffer: &[u8] = &[0u8; 0];
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM1);
        let dma2 = &*stm32::DMA2::ptr();
        half = dma2.lisr.read().bits() & (1 << (HTIF_OFFSET + STREAM1_OFFSET)) > 0;
        dma2.lifcr.write(|w| w.bits(0x3D << STREAM1_OFFSET));
        let address = dma2.st[1].m0ar.read().bits();
        let size = *((address - 2) as *const u16) as usize;
        buffer = core::slice::from_raw_parts(address as *const _, size);
    });
    if let Some(ref mut device) = DEVICE {
        device.handle(buffer, half);
    }
}

pub fn init(
    usart6: stm32::USART6,
    pins: (PC6, PC7),
    nvic: &mut cortex_m::peripheral::NVIC,
    config: &SerialConfig,
    clocks: Clocks,
) -> Option<&'static mut Device> {
    let (pc6, pc7) = pins;
    let pins = (pc6.into_alternate_af8(), pc7.into_alternate_af8());
    Serial::usart6(usart6, pins, to_serial_config(&config), clocks).unwrap();

    // dma2 stream1 channel 5 rx
    unsafe {
        let usart = &*stm32::USART6::ptr();
        usart.cr3.modify(|_, w| w.dmar().enabled());

        let dma_buffer = Box::leak(alloc_by_config(&config));
        let address = dma_buffer.as_ptr() as usize + 2;
        let size = dma_buffer.len() - 2;
        *(dma_buffer as *mut _ as *mut u16) = size as u16;
        debug!("Alloc DMA buffer at {:#X} size {} on USART6", address, size);

        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[1];
        stream.ndtr.write(|w| w.ndt().bits(size as u16));
        stream.par.write(|w| w.pa().bits(&usart.dr as *const _ as u32));
        stream.m0ar.write(|w| w.m0a().bits(address as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(5).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .pl().very_high().htie().enabled().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM1);
    unsafe {
        cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM1);
    }

    let device = match config {
        SerialConfig::SBUS(_) => {
            unsafe { nvic.set_priority(stm32::Interrupt::DMA2_STREAM1, 18) }; // No less than DMA1 stream0
            Device::SBUS(SbusReceiver::new())
        }
        _ => return None,
    };
    unsafe { DEVICE = Some(device) }
    unsafe { DEVICE.as_mut() }
}
