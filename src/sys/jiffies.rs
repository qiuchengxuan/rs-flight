use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;

static mut JIFFIES: AtomicUsize = AtomicUsize::new(0);
static mut RATE: usize = 200;

pub fn get() -> Duration {
    unsafe {
        let jiffies = JIFFIES.load(Ordering::Relaxed);
        let seconds = jiffies / RATE;
        let nanos = (jiffies % RATE) * (1000 / RATE) * 1000_000;
        Duration::new(seconds as u64, nanos as u32)
    }
}

fn tick() {
    unsafe { JIFFIES.fetch_add(1, Ordering::Relaxed) };
}

pub fn init(rate: usize) -> fn() {
    assert!(0 < rate && rate <= 1000);
    unsafe { RATE = rate }
    tick
}
