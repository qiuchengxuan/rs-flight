#![no_std]

#[macro_use]
extern crate cortex_m_rt;
extern crate cortex_m;
#[cfg(feature = "stm32")]
extern crate drone_cortexm;
#[cfg(feature = "stm32")]
extern crate drone_stm32_map;
#[cfg(feature = "stm32")]
extern crate stm32f4xx_hal;

pub mod cortex_m4;
#[cfg(feature = "stm32")]
pub mod stm32f4;
