use alloc::rc::Rc;
use core::mem::MaybeUninit;

use stm32f4xx_hal::adc::config::{AdcConfig, Continuous, Dma, SampleTime, Sequence};
use stm32f4xx_hal::adc::Adc;
use stm32f4xx_hal::gpio::gpioc::PC2;
use stm32f4xx_hal::gpio::Floating;
use stm32f4xx_hal::gpio::Input;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::stm32;

use pro_flight::algorithm::lpf::LPF;
use pro_flight::datastructures::data_source::u16_source::{U16Data, U16DataSource};
use pro_flight::datastructures::data_source::DataWriter;
use pro_flight::datastructures::measurement::battery::Battery;

const VOLTAGE_SCALE_X100: usize = 1100;
const SAMPLE_SIZE: usize = 16;
const VREF: usize = 3300;

pub struct Adc2VBat {
    adc: Adc<stm32::ADC2>,
    lpf: LPF<u16>,
    vbat_data: Rc<U16Data<Battery>>,
    dma_buffer: [u16; SAMPLE_SIZE],
}

impl Adc2VBat {
    pub fn new(adc: Adc<stm32::ADC2>) -> Self {
        let vbat_data = Rc::new(U16Data::default());
        let lpf = LPF::<u16>::new(1.0, 5.0);
        Self { adc, lpf, vbat_data, dma_buffer: Default::default() }
    }

    pub fn reader(&self) -> U16DataSource<Battery> {
        U16DataSource::new(&self.vbat_data)
    }

    fn dma_rx_done(&mut self) {
        let sum: usize = self.dma_buffer.iter().map(|&v| v as usize).sum();
        let value = self.lpf.filter((sum / SAMPLE_SIZE * VREF / 0xFFF) as u16) as usize;
        let milli_voltages = value * VOLTAGE_SCALE_X100 / 100;
        self.vbat_data.write(Battery(milli_voltages as u16));
    }
}

static mut ADC2_VBAT: MaybeUninit<Adc2VBat> = MaybeUninit::uninit();

#[interrupt]
unsafe fn DMA2_STREAM2() {
    cortex_m::interrupt::free(|_| {
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM2);
        { &*stm32::DMA2::ptr() }.lifcr.write(|w| w.bits(0x3D << 16));
    });

    (&mut *ADC2_VBAT.as_mut_ptr()).dma_rx_done();
}

pub fn init(adc2: stm32::ADC2, pc2: PC2<Input<Floating>>) -> &'static Adc2VBat {
    let config = AdcConfig::default().dma(Dma::Continuous).continuous(Continuous::Continuous);

    let mut adc = Adc::adc2(adc2, true, config);
    let vbat = pc2.into_analog();
    adc.configure_channel(&vbat, Sequence::One, SampleTime::Cycles_480);
    adc.start_conversion();
    unsafe { ADC2_VBAT = MaybeUninit::new(Adc2VBat::new(adc)) };
    let adc2_vbat = unsafe { &mut *ADC2_VBAT.as_mut_ptr() };

    // dma2 stream2 channel 1 rx
    unsafe {
        let dma2 = &*(stm32::DMA2::ptr());
        let stream = &dma2.st[2];
        stream.ndtr.write(|w| w.ndt().bits(adc2_vbat.dma_buffer.len() as u16));
        stream.par.write(|w| w.pa().bits(adc2_vbat.adc.data_register_address()));
        let m0ar = &stream.m0ar;
        m0ar.write(|w| w.m0a().bits(adc2_vbat.dma_buffer.as_ptr() as u32));
        #[rustfmt::skip]
        stream.cr.write(|w| {
            w.chsel().bits(1).minc().incremented().dir().peripheral_to_memory().circ().enabled()
                .msize().bits16().psize().bits16().pl().high().tcie().enabled().en().enabled()
        });
    }

    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::DMA2_STREAM2);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::DMA2_STREAM2) }
    adc2_vbat
}
