use drone_core::fib::{new_fn, FiberStreamPulse, ThrFiberStreamPulse, Yielded};
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::sys_tick::SysTickPeriph;
use pro_flight::sys::jiffies;

use super::clock::SYSCLK;

pub fn init<T>(systick: SysTickPeriph, thread: T, ticks_per_second: usize) -> FiberStreamPulse
where
    T: ThrFiberStreamPulse,
{
    let jiffies_callback = jiffies::init(ticks_per_second);
    let stream = thread.add_saturating_pulse_stream(new_fn(move || {
        jiffies_callback();
        Yielded(Some(1))
    }));
    systick.stk_val.store(|r| r.write_current(0));
    systick.stk_load.store(|r| r.write_reload(SYSCLK / ticks_per_second as u32 - 1));
    systick.stk_ctrl.store(|r| r.set_clksource().set_tickint().set_enable());
    stream
}
