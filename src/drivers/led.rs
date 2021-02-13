use core::time::Duration;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::timer::CountDown;

pub struct LED<P, T> {
    pin: P,
    timer: T,
    on: bool,
}

impl<E, P: OutputPin<Error = E>, T: CountDown<Time = Duration>> LED<P, T> {
    pub fn new(mut pin: P, timer: T) -> Self {
        pin.set_low().ok();
        Self { pin, timer, on: true }
    }

    pub fn check_toggle(&mut self) {
        if !self.timer.wait().is_ok() {
            return;
        }
        if self.on {
            self.timer.start(Duration::from_millis(980));
            self.pin.set_high().ok();
        } else {
            self.timer.start(Duration::from_millis(20));
            self.pin.set_low().ok();
        }
        self.on = !self.on;
    }
}
