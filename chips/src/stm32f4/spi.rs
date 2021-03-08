pub struct BaudrateControl(pub u32);

impl BaudrateControl {
    pub fn new(pclk: u32, freq: u32) -> Self {
        Self(match pclk / freq {
            0 => unreachable!(),
            1..=2 => 0b000,
            3..=5 => 0b001,
            6..=11 => 0b010,
            12..=23 => 0b011,
            24..=47 => 0b100,
            48..=95 => 0b101,
            96..=191 => 0b110,
            _ => 0b111,
        })
    }
}

#[macro_export]
macro_rules! __define_spi {
    ($spi:ident => (
        $gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident
    )) => {
        type $sclk = gpio::$gpio::$sclk<Input<Floating>>;
        type $miso = gpio::$gpio::$miso<Input<Floating>>;
        type $mosi = gpio::$gpio::$mosi<Input<Floating>>;

        pub struct $spi<INT, SPI: SpiMap> {
            sr: SPI::CSpiSr,
            dr: SPI::CSpiDr,
            int: INT,
            sclk: gpio::$gpio::$sclk<Alternate<gpio::$af>>,
            miso: gpio::$gpio::$miso<Alternate<gpio::$af>>,
            mosi: gpio::$gpio::$mosi<Alternate<gpio::$af>>,
        }

        impl<INT: ThrNvic> $spi<INT, spi::$spi> {
            pub fn new(
                regs: SpiPeriph<spi::$spi>,
                pins: ($sclk, $miso, $mosi),
                int: INT,
                baudrate: $crate::stm32f4::spi::BaudrateControl,
                mode: Mode,
            ) -> Self {
                let (sclk, miso, mosi) = pins;
                regs.rcc_busenr_spien.set_bit();
                regs.spi_cr1.modify(|r| {
                    if mode.polarity == Polarity::IdleHigh {
                        r.set_cpol();
                    }
                    if mode.phase == Phase::CaptureOnSecondTransition {
                        r.set_cpha();
                    }
                    r.write_br(baudrate.0)
                });
                regs.spi_cr2.store(|r| {
                    r.set_rxneie().set_errie().set_frf().set_ssoe();
                    r.set_txdmaen().set_rxdmaen()
                });
                let (sclk, miso, mosi) = (sclk.$into_af(), miso.$into_af(), mosi.$into_af());
                let (sr, dr) = (regs.spi_sr.into_copy(), regs.spi_dr.into_copy());
                Self { sr, dr, int, sclk, miso, mosi }
            }
        }

        impl<INT: ThrNvic> embedded_hal::blocking::spi::Transfer<u8> for $spi<INT, spi::$spi> {
            type Error = Error;

            fn transfer<'a>(&mut self, bytes: &'a mut [u8]) -> Result<&'a [u8], Error> {
                let (sr, dr) = (self.sr, self.dr);
                self.int.enable_int();
                for i in 0..bytes.len() {
                    let future = self.int.add_future(fib::new_fn(move || {
                        let status = sr.load();
                        let result = match () {
                            _ if status.ovr() => Err(Error::Overrun),
                            _ if status.modf() => Err(Error::ModeFault),
                            _ if status.crcerr() => Err(Error::Crc),
                            _ if status.rxne() => Ok(dr.load().dr()),
                            _ => return fib::Yielded(()),
                        };
                        fib::Complete(result)
                    }));
                    dr.store(|r| r.write_dr(bytes[i] as u32));
                    bytes[i] = future.root_wait()? as u8;
                }
                self.int.disable_int();
                Ok(bytes)
            }
        }
    };
}

#[macro_export]
macro_rules! __define_spis {
    () => {};
    (
        $spi:ident => (
            $gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident
        )
        $($spis:ident => (
            $gpios:ident, $sclks:ident, $misos:ident, $mosis:ident, $afs:ident, $into_afs:ident
        ))*
    ) => {
        __define_spi!{$spi => ($gpio, $sclk, $miso, $mosi, $af, $into_af)}
        __define_spis!{$($spis => ($gpios, $sclks, $misos, $mosis, $afs, $into_afs))*}
    }
}

#[macro_export]
macro_rules! define_spis {
    ($($spi:ident => (
        $gpio:ident, $sclk:ident, $miso:ident, $mosi:ident, $af:ident, $into_af:ident
    ))+) => {
        use drone_core::sync::spsc::oneshot;
        use drone_core::fib::FiberFuture;
        use drone_cortexm::{fib, reg::prelude::*, thr::prelude::*, thr::ThrNvic};
        use drone_stm32_map::periph::spi::{self, traits::*, SpiPeriph, SpiMap};
        use embedded_hal::spi::{Mode, Phase, Polarity};
        use stm32f4xx_hal::gpio::{self, Alternate, Floating, Input, Output, PullUp, PushPull};

        #[derive(Copy, Clone)]
        pub enum Error {
            Overrun,
            ModeFault,
            Crc,
        }

        __define_spis!{$($spi => ($gpio, $sclk, $miso, $mosi, $af, $into_af))+}
    };
}
