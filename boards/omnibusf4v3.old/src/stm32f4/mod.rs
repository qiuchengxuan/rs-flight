use alloc::boxed::Box;

use pro_flight::config::SerialConfig;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::config::{Config, DmaConfig, Parity, StopBits, WordLength};

pub fn to_serial_config(config: &SerialConfig) -> Config {
    match config {
        SerialConfig::GNSS(gnss) => {
            Config { baudrate: gnss.baudrate.bps(), dma: DmaConfig::Rx, ..Default::default() }
        }
        SerialConfig::SBUS(sbus) => Config {
            baudrate: sbus.baudrate().bps(),
            stopbits: StopBits::STOP2,
            parity: Parity::ParityEven,
            wordlength: WordLength::DataBits9,
            dma: DmaConfig::TxRx,
        },
    }
}

pub fn alloc_by_config(config: &SerialConfig) -> Box<[u8]> {
    match config {
        SerialConfig::GNSS(_) => Box::new([0u8; 128 + 2]),
        SerialConfig::SBUS(_) => Box::new([0u8; 64 + 2]),
    }
}
