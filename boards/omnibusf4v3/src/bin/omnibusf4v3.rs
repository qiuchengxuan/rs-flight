#![no_main]
#![no_std]

use drone_core::{mem, token::Token};
use drone_cortexm::processor;
use omnibusf4v3::thread::{ThrsInit, Vtable};
use omnibusf4v3::{tasks, Regs};

/// The vector table.
#[no_mangle]
pub static VTABLE: Vtable = Vtable::new(reset);

/// The entry point.
///
/// # Safety
///
/// This function should not be called by software.
#[no_mangle]
pub unsafe extern "C" fn reset() -> ! {
    mem::bss_init();
    mem::data_init();
    processor::fpu_init(true);
    tasks::root(Regs::take(), ThrsInit::take());
    loop {
        processor::wait_for_int();
    }
}
