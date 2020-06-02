#![no_std]
#![feature(maybe_uninit_uninit_array)]

extern crate ahrs;
extern crate arrayvec;
extern crate ascii;
#[macro_use]
extern crate ascii_osd_hud;
extern crate btoi;
#[macro_use]
extern crate enum_map;
extern crate integer_sqrt;
extern crate micromath;
#[macro_use]
extern crate mpu6000;
extern crate nalgebra;
extern crate nb;

#[macro_use]
pub mod components;

pub mod datastructures;
pub mod drivers;
pub mod hal;

#[cfg(test)]
extern crate std;
