#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt;
#[cfg(feature = "stm32")]
extern crate drone_cortexm;
#[cfg(feature = "stm32")]
extern crate drone_stm32_map;
#[cfg(feature = "stm32")]
extern crate stm32f4xx_hal;

#[cfg(feature = "stm32")]
pub mod stm32f4;
