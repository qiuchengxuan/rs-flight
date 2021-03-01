use alloc::boxed::Box;
use alloc::vec::Vec;

use embedded_hal::PwmPin;

use crate::components::event::OnEvent;
use crate::components::mixer::ControlMixer;
use crate::config;
use crate::config::aircraft::Configuration;
use crate::config::peripherals::pwm::{ServoType, PWM as PwmConfig};
use crate::datastructures::data_source::StaticData;
use crate::datastructures::input::ControlInput;

use super::pwm::{to_motor_pwm_duty, to_servo_pwm_duty};

type PWM = Box<dyn PwmPin<Duty = u16>>;

pub struct FixedWing<S> {
    mixer: ControlMixer<S>,
    pwms: Vec<(&'static str, PWM)>,
    configs: Vec<Option<PwmConfig>>,
    config_iteration: u8,
}

impl<S> FixedWing<S> {
    pub fn new(mixer: ControlMixer<S>, pwms: Vec<(&'static str, PWM)>) -> Self {
        let config_iteration = config::get().iteration().wrapping_sub(1);
        let configs = vec![None; pwms.len()];
        Self { mixer, pwms, configs, config_iteration }
    }

    fn reload_config(&mut self) {
        let config = &config::get().peripherals.pwms;
        for (i, (name, _)) in self.pwms.iter().enumerate() {
            self.configs[i] = config.get(name).map(|pwm| pwm.clone());
        }
    }
}

impl<S: StaticData<ControlInput>> OnEvent for FixedWing<S> {
    fn on_event(&mut self) {
        if self.config_iteration != config::get().iteration() {
            self.reload_config();
        }
        let input = self.mixer.mix();
        let (left, right) = match config::get().aircraft.configuration {
            Configuration::FlyingWing => (-input.roll + input.pitch, input.roll + input.pitch),
            Configuration::VTail => (input.yaw + input.pitch, -input.yaw + input.pitch),
            _ => (0, 0),
        };
        for (i, (_, pwm)) in self.pwms.iter_mut().enumerate() {
            let config = match self.configs[i] {
                Some(config) => config,
                None => continue,
            };
            let max_duty = pwm.get_max_duty();
            let duty = match config {
                PwmConfig::Motor(_) => to_motor_pwm_duty(max_duty, config.rate(), input.throttle),
                PwmConfig::Servo(servo) => {
                    let axis = match servo.servo_type {
                        ServoType::Aileron => input.roll,
                        ServoType::Elevator => input.pitch,
                        ServoType::Rudder => input.yaw,
                        ServoType::ElevonLeft => left,
                        ServoType::ElevonRight => right,
                    };
                    let (min, max) = (servo.min_angle, servo.max_angle);
                    to_servo_pwm_duty(max_duty, axis, min, max, servo.reversed)
                }
            };
            pwm.set_duty(duty);
        }
    }
}
