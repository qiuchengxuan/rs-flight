use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::sys_tick::SysTickPeriph;

use super::clock::SYSCLK;

pub fn init(systick: SysTickPeriph, rate: usize) {
    systick.stk_val.store(|r| r.write_current(0));
    systick.stk_load.store(|r| r.write_reload(SYSCLK / rate as u32 - 1));
    systick.stk_ctrl.store(|r| r.set_clksource().set_tickint().set_enable());
}
