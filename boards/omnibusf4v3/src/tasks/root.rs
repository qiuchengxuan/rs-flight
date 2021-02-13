//! The root task.

use chips::{stm32f4::dfu::Dfu, stm32f4::valid_memory_address};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::sys_tick::periph_sys_tick;
use futures::prelude::*;
use pro_flight::components::cli::{memory, Command, CLI};
use pro_flight::drivers::led::LED;
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

fn reboot() {
    cortex_m::peripheral::SCB::sys_reset()
}

fn bootloader() {
    unsafe { DFU.arm() };
    cortex_m::peripheral::SCB::sys_reset()
}

static mut DFU: Dfu = Dfu(0);

/// The root task handler.
#[inline(never)]
pub fn handler(reg: Regs, thr_init: ThrsInit) {
    unsafe { DFU.check() };

    memory::init(valid_memory_address);
    let thread = thread::init(thr_init);
    thread.hard_fault.add_once(|| panic!("Hard Fault"));

    reg.rcc_apb1enr.pwren.set_bit();
    let regs = (reg.rcc_cfgr, reg.rcc_cir, reg.rcc_cr, reg.rcc_pllcfgr, reg.flash_acr);
    stm32f4::clock::setup(thread.rcc, regs).root_wait();
    let mut stream = stm32f4::systick::init(periph_sys_tick!(reg), thread.sys_tick, TICKS_PER_SEC);

    let peripherals = stm32::Peripherals::take().unwrap();
    let gpio_b = peripherals.GPIOB.split();
    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), SysTimer::new());

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

    let commands = [
        Command::new("reboot", "Reboot", |_| reboot()),
        Command::new("bootloader", "Reboot in bootloader", |_| bootloader()),
    ];
    let mut cli = CLI::new(&commands);
    while let Some(_) = stream.next().root_wait() {
        poller.poll(|bytes| cli.receive(bytes));
        led.check_toggle();
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
