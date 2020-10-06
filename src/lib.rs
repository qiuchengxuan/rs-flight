#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]

#[macro_use]
extern crate alloc;
extern crate ascii;
extern crate ascii_osd_hud;
extern crate bmp280_core as bmp280;
extern crate crc;
extern crate embedded_dma;
extern crate embedded_sdmmc;
#[macro_use]
extern crate enum_map;
extern crate integer_sqrt;
extern crate micromath;
#[macro_use]
extern crate mpu6000;
extern crate nalgebra;
extern crate nb;
extern crate nmea0183_core as nmea0183;
extern crate qmc5883l;
extern crate sbus_parser;
#[macro_use]
extern crate sval;
extern crate sval_json;
extern crate usb_device;
extern crate usbd_serial;

#[macro_use]
pub mod sys;
#[macro_use]
pub mod components;
pub mod algorithm;
pub mod config;
pub mod datastructures;
pub mod drivers;
pub mod hal;

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[cfg(test)]
#[macro_use]
extern crate serial_test;

#[cfg(test)]
#[macro_use]
extern crate hex_literal;
