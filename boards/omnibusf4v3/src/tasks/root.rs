//! The root task.

use chips::stm32f4::{
    clock,
    dfu::Dfu,
    flash::{Flash, Sector},
    rtc, systick, usb_serial, valid_memory_address,
};
use drone_cortexm::{reg::prelude::*, thr::prelude::*};
use drone_stm32_map::periph::{flash::periph_flash, rtc::periph_rtc, sys_tick::periph_sys_tick};
use futures::prelude::*;
use pro_flight::{
    components::{
        cli::{memory, Command, CLI},
        logger::{self, Level},
    },
    config,
    drivers::led::LED,
    drivers::nvram::NVRAM,
    sys::{jiffies, time, timer},
};
use stm32f4xx_hal::{
    otg_fs::{UsbBus, USB},
    prelude::*,
    stm32,
};

use crate::{
    flash::FlashWrapper,
    rtc::{RTCReader, RTCWriter},
    thread,
    thread::ThrsInit,
    Regs,
};

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
    let mut thread = thread::init(thr_init);
    thread.hard_fault.add_once(|| panic!("Hard Fault"));
    thread.rcc.enable_int();
    let rcc_cir = reg.rcc_cir.into_copy();

    reg.rcc_apb1enr.pwren.set_bit();
    let regs = (reg.rcc_cfgr, reg.rcc_cr, reg.rcc_pllcfgr);
    clock::setup_pll(&mut thread.rcc, rcc_cir, regs, &reg.flash_acr).root_wait();
    let callback = jiffies::init(TICKS_PER_SEC);
    let mut stream = systick::init(periph_sys_tick!(reg), thread.sys_tick, TICKS_PER_SEC, callback);

    let peripherals = stm32::Peripherals::take().unwrap();
    let gpio_b = peripherals.GPIOB.split();
    let mut led = LED::new(gpio_b.pb5.into_push_pull_output(), timer::SysTimer::new());

    reg.pwr_cr.modify(|r| r.set_dbp());
    reg.rcc_bdcr.modify(|r| r.set_rtcsel1().set_rtcsel0().set_rtcen()); // select HSE
    let mut rtc = rtc::RTC::new(periph_rtc!(reg));
    rtc.disable_write_protect();
    let rtc_reader = rtc.reader();
    time::init(RTCWriter(rtc), RTCReader(rtc_reader));

    logger::init(Level::Debug);

    let gpio_a = peripherals.GPIOA.split();
    let usb = USB {
        usb_global: peripherals.OTG_FS_GLOBAL,
        usb_device: peripherals.OTG_FS_DEVICE,
        usb_pwrclk: peripherals.OTG_FS_PWRCLK,
        pin_dm: gpio_a.pa11.into_alternate_af10(),
        pin_dp: gpio_a.pa12.into_alternate_af10(),
        hclk: clock::HCLK.into(),
    };
    let allocator = UsbBus::new(usb, Box::leak(Box::new([0u32; 1024])));
    let mut poller = usb_serial::init(allocator);

    let flash = FlashWrapper::new(Flash::new(periph_flash!(reg)));
    let sector1 = unsafe { Sector::new(1).unwrap().as_slice() };
    let sector2 = unsafe { Sector::new(2).unwrap().as_slice() };
    let mut nvram = NVRAM::new(flash, [sector1, sector2]).unwrap();
    match nvram.load() {
        Ok(config) => config::replace(config),
        Err(error) => error!("Load config failed: {:?}", error),
    }

    let mut commands = [
        Command::new("reboot", "Reboot", |_| reboot()),
        Command::new("bootloader", "Reboot in bootloader", |_| bootloader()),
        Command::new("logread", "Show log", |_| println!("{}", logger::get())),
        Command::new("save", "Save configuration", move |_| {
            if let Some(err) = nvram.store(config::get()).err() {
                println!("Save configuration failed: {:?}", err);
                nvram.reset().ok();
            }
        }),
    ];
    let mut cli = CLI::new(&mut commands);
    while let Some(_) = stream.next().root_wait() {
        poller.poll(|bytes| cli.receive(bytes));
        led.check_toggle();
    }

    reg.scb_scr.sleeponexit.set_bit(); // Enter a sleep state on ISR exit.
}
