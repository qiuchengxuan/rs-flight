use drone_core::fib::{new_fn, FiberStreamPulse, ThrFiberStreamPulse, Yielded};
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::sys_tick::SysTickPeriph;

use super::clock::SYSCLK;

pub fn init<T>(systick: SysTickPeriph, thread: T, rate: usize, callback: fn()) -> FiberStreamPulse
where
    T: ThrFiberStreamPulse,
{
    let stream = thread.add_saturating_pulse_stream(new_fn(move || {
        callback();
        Yielded(Some(1))
    }));
    systick.stk_val.store(|r| r.write_current(0));
    systick.stk_load.store(|r| r.write_reload(SYSCLK / rate as u32 - 1));
    systick.stk_ctrl.store(|r| r.set_clksource().set_tickint().set_enable());
    stream
}
