use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU32, Ordering};
use core::time::Duration;

use drone_core::fib::{new_fn, Yielded};
use drone_core::thr::ThrToken;
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::sys_tick::SysTickPeriph;

use super::clock::SYSCLK;

pub const RATE: u32 = 1000;
static mut SYS_TICK_PERIPH: MaybeUninit<SysTickPeriph> = MaybeUninit::uninit();
static COUNTER: AtomicU32 = AtomicU32::new(0);

#[no_mangle]
fn get_jiffies() -> Duration {
    cortex_m::interrupt::free(|_| {
        let systick = unsafe { &mut *SYS_TICK_PERIPH.as_mut_ptr() };
        let ns = systick.stk_val.current.read_bits() * 1000 / (SYSCLK / RATE / 1000);
        let counter = COUNTER.load(Ordering::Relaxed);
        let us = counter % 1000;
        Duration::new((counter / RATE) as u64, us * 1000_000 + ns)
    })
}

pub fn init(systick: SysTickPeriph, thread: impl ThrToken) {
    systick.stk_val.store(|r| r.write_current(0));
    systick.stk_load.store(|r| r.write_reload(SYSCLK / RATE - 1));
    systick.stk_ctrl.store(|r| r.set_clksource().set_tickint().set_enable());
    unsafe { SYS_TICK_PERIPH = MaybeUninit::new(systick) };
    thread.add_fib(new_fn(move || {
        COUNTER.fetch_add(1, Ordering::Relaxed);
        Yielded::<(), ()>(())
    }));
}
