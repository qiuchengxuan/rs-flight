use alloc::boxed::Box;

use pro_flight::components::event::{Notify, OnEvent};
use stm32f4xx_hal::interrupt;
use stm32f4xx_hal::stm32;

static mut LISTENER: Option<Box<dyn OnEvent>> = None;

#[interrupt]
unsafe fn EXTI0() {
    let mut swier = 0;
    cortex_m::interrupt::free(|_| {
        let exti = &*stm32::EXTI::ptr();
        swier = exti.swier.read().bits();
        exti.pr.write(|w| w.bits(swier));
        cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI0);
    });
    if let Some(ref mut listener) = LISTENER {
        listener.on_event();
    }
}

#[derive(Copy, Clone)]
pub struct EventTrigger;

impl Notify for EventTrigger {
    fn notify(&mut self) {
        unsafe {
            let exti = &*stm32::EXTI::ptr();
            exti.swier.write(|w| w.bits(1));
        }
    }
}

pub fn init(exti: &mut stm32::EXTI, listener: Box<dyn OnEvent>) -> EventTrigger {
    unsafe { LISTENER = Some(listener) }
    exti.imr.modify(|r, w| unsafe { w.bits(r.bits() | 1) });
    cortex_m::peripheral::NVIC::unpend(stm32::Interrupt::EXTI0);
    unsafe { cortex_m::peripheral::NVIC::unmask(stm32::Interrupt::EXTI0) }
    EventTrigger
}
