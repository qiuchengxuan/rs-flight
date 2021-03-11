use core::time::Duration;

extern "Rust" {
    fn get_jiffies() -> Duration;
}

pub fn get() -> Duration {
    unsafe { get_jiffies() }
}
