use embedded_hal::blocking::delay::DelayUs;
use mpu6000::bus::Bus;
use mpu6000::registers::{AccelerometerSensitive, GyroSensitive};
use mpu6000::{self, ClockSource, IntPinConfig, Interrupt, MPU6000};

use crate::config;
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::{Acceleration, Axes, Measurement};
use crate::drivers::{accelerometer, gyroscope};
use crate::sys::timer::SysTimer;

static mut ACCELEROMETER_SENSITIVE: AccelerometerSensitive =
    accelerometer_sensitive!(+/-16g, 2048/LSB);
pub const GYRO_SENSITIVE: GyroSensitive = gyro_sensitive!(+/-1000dps, 32.8LSB/dps);

impl Into<Measurement> for mpu6000::Acceleration {
    fn into(self) -> Measurement {
        let axes = Axes { x: -self.0 as i32, y: -self.1 as i32, z: -self.2 as i32 };
        let sensitive: f32 = unsafe { ACCELEROMETER_SENSITIVE }.into();
        Measurement { axes, sensitive: sensitive as i32 }
    }
}

impl Into<Measurement> for mpu6000::Gyro {
    fn into(self) -> Measurement {
        let axes =
            Axes { x: (self.0 as i32) << 8, y: (self.1 as i32) << 8, z: (self.2 as i32) << 8 };
        let sensitive: f32 = GYRO_SENSITIVE.into();
        Measurement { axes, sensitive: (sensitive * 256.0) as i32 }
    }
}

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 16]) {
    let buf: &[i16; 8] = core::mem::transmute(dma_buffer);
    let acceleration: mpu6000::Acceleration = buf[1..4].into();
    let gyro: mpu6000::Gyro = buf[5..].into();
    if let Some(ref mut accelerometer) = accelerometer::ACCELEROMETER {
        accelerometer.write(Acceleration(acceleration.into()));
    }
    if let Some(ref mut gyroscope) = gyroscope::GYROSCOPE {
        gyroscope.write(gyro.into());
    }
}

pub fn init<E>(bus: impl Bus<Error = E>, sample_rate: u16) -> Result<bool, E> {
    let mut mpu6000 = MPU6000::new(bus);
    let mut delay = SysTimer::new();
    mpu6000.reset(&mut delay)?;
    if !mpu6000.verify()? {
        return Ok(false);
    }
    info!("MPU6000 detected");
    mpu6000.set_sleep(false)?;
    delay.delay_us(15u8);
    mpu6000.set_i2c_disable(true)?;
    delay.delay_us(15u8);
    mpu6000.set_clock_source(ClockSource::PLLGyroZ)?;
    delay.delay_us(15u8);
    let imu = config::get().imu;
    let sensitive = match imu.accelerometer.sensitive.integer() {
        0..=2 => accelerometer_sensitive!(+/-2g, 16384/LSB),
        3..=4 => accelerometer_sensitive!(+/-4g, 8192/LSB),
        5..=8 => accelerometer_sensitive!(+/-8g, 4096/LSB),
        _ => accelerometer_sensitive!(+/-16g, 2048/LSB),
    };
    mpu6000.set_accelerometer_sensitive(sensitive)?;
    unsafe { ACCELEROMETER_SENSITIVE = sensitive };
    delay.delay_us(15u8);
    mpu6000.set_gyro_sensitive(GYRO_SENSITIVE)?;
    delay.delay_us(15u8);
    mpu6000.set_dlpf(2)?;
    delay.delay_us(15u8);
    mpu6000.set_sample_rate(sample_rate)?;
    delay.delay_us(15u8);
    mpu6000.set_int_pin_config(IntPinConfig::IntReadClear, true)?;
    delay.delay_us(15u8);
    mpu6000.set_interrupt_enable(Interrupt::DataReady, true)?;
    Ok(true)
}
