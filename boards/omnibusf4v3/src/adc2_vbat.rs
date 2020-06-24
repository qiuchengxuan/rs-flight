use core::mem::MaybeUninit;

use stm32f4xx_hal::adc::config::{AdcConfig, Continuous, Dma, SampleTime, Sequence};
use stm32f4xx_hal::adc::Adc;
use stm32f4xx_hal::gpio::gpioc::PC2;
use stm32f4xx_hal::gpio::Floating;
use stm32f4xx_hal::gpio::Input;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::stm32;

use rs_flight::datastructures::U16DataSource;

const VOLTAGE_SCALE_X100: usize = 1100;
const SAMPLE_SIZE: usize = 16;
const VREF: usize = 3300;

static mut DMA_BUFFER: [u16; SAMPLE_SIZE] = [0u16; SAMPLE_SIZE];
static mut VBAT_EVENT: MaybeUninit<U16DataSource> = MaybeUninit::uninit();
static mut ADC2: MaybeUninit<Adc<stm32::ADC2>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn DMA2_STREAM2() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM2);
        let dma2 = &*stm32::DMA2::ptr();
        dma2.lifcr.write(|w| w.bits(0x3D << 16));
    });

    let buf = &DMA_BUFFER;
    let sum: usize = buf.iter().map(|&v| v as usize).sum();
    let milli_voltages = (sum / SAMPLE_SIZE) * VREF / 0xFFF * VOLTAGE_SCALE_X100 / 100;
    { &mut *VBAT_EVENT.as_mut_ptr() }.write(milli_voltages as u16);
}

pub fn init(adc2: stm32::ADC2, pc2: PC2<Input<Floating>>) -> &'static U16DataSource {
    let config = AdcConfig::default().dma(Dma::Continuous).continuous(Continuous::Continuous);
    let mut adc = Adc::adc2(adc2, true, config);

    // dma2 stream2 channel 1 rx
    unsafe {
        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[2];
        stream.ndtr.write(|w| w.ndt().bits(DMA_BUFFER.len() as u16));
        stream.par.write(|w| w.pa().bits(adc.data_register_address()));
        let m0ar = &stream.m0ar;
        m0ar.write(|w| w.m0a().bits(DMA_BUFFER.as_ptr() as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(1).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .msize().bits16().psize().bits16().pl().high().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM2);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM2) }
    let vbat = pc2.into_analog();
    adc.configure_channel(&vbat, Sequence::One, SampleTime::Cycles_480);
    adc.start_conversion();
    unsafe {
        ADC2 = MaybeUninit::new(adc);
        &*VBAT_EVENT.as_ptr()
    }
}