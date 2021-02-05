use alloc::boxed::Box;
use core::mem::MaybeUninit;

use pro_flight::components::schedule::Schedulable;
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::timer::{Event, Timer};
use stm32f4xx_hal::{prelude::*, stm32};

static mut SCHEDULER: Option<Box<dyn Schedulable>> = None;
static mut TIM7: MaybeUninit<Timer<stm32::TIM7>> = MaybeUninit::uninit();

#[interrupt]
unsafe fn TIM7() {
    cortex_m::interrupt::free(|_| {
        (&mut *TIM7.as_mut_ptr()).clear_interrupt(Event::TimeOut);
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM7);
    });
    if let Some(ref mut scheduler) = SCHEDULER {
        scheduler.schedule();
    }
}

pub fn init(tim7: stm32::TIM7, scheduler: Box<dyn Schedulable>, clocks: Clocks, rate: u32) {
    let mut timer = Timer::tim7(tim7, rate.hz(), clocks);
    timer.listen(Event::TimeOut);
    unsafe { TIM7 = MaybeUninit::new(timer) };
    unsafe { SCHEDULER = Some(scheduler) }
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::TIM7);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::TIM7) }
}
