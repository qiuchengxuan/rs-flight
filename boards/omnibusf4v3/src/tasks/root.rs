//! The root task.

use core::time::Duration;

use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::sys_tick::periph_sys_tick;
use embedded_hal::timer::CountDown;
use futures::prelude::*;
use pro_flight::drivers::terminal::Terminal;
use pro_flight::sys::timer::SysTimer;
use stm32f4xx_hal::{
    otg_fs::{UsbBus, USB},
    prelude::*,
    stm32,
};

use crate::stm32f4;
use crate::stm32f4::usb_serial;
use crate::{thread, thread::ThrsInit, Regs};

const TICKS_PER_SEC: usize = 200;

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    let thread = thread::init(thr_init);

    thread.hard_fault.add_once(|| panic!("Hard Fault"));

    reg.rcc_apb1enr.pwren.set_bit();
    let regs = (reg.rcc_cfgr, reg.rcc_cir, reg.rcc_cr, reg.rcc_pllcfgr, reg.flash_acr);
    stm32f4::clock::setup(thread.rcc, regs).root_wait();
    let mut stream = stm32f4::systick::init(periph_sys_tick!(reg), thread.sys_tick, TICKS_PER_SEC);

    let peripherals = stm32::Peripherals::take().unwrap();
    let gpio_b = peripherals.GPIOB.split();
    let mut led = gpio_b.pb5.into_push_pull_output();
    led.set_low().ok();

    let gpio_a = peripherals.GPIOA.split();
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
        hclk: stm32f4::clock::HCLK.into(),
    };

    let allocator = UsbBus::new(usb, Box::leak(Box::new([0u32; 1024])));
    let mut poller = usb_serial::init(allocator);
    let mut terminal = Terminal::new();

    let mut timer = SysTimer::new();
    let mut on = false;
    while let Some(_) = stream.next().root_wait() {
        poller.poll(|bytes| {
            terminal.receive(bytes);
        });
        if !timer.wait().is_ok() {
            continue;
        }
        if on {
            timer.start(Duration::from_millis(980));
            led.set_high().ok();
        } else {
            timer.start(Duration::from_millis(20));
            led.set_low().ok();
        }
        on = !on;
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
