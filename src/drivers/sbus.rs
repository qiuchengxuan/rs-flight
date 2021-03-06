use alloc::boxed::Box;
use alloc::rc::Rc;

use sbus_parser::{is_sbus_packet_end, SbusData, SbusPacket, SBUS_PACKET_BEGIN, SBUS_PACKET_SIZE};

use crate::components::event::Notify;
use crate::config;
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::u16_source::{U16Data, U16DataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::input::{ControlInput, InputType, RSSI};

pub struct SbusReceiver {
    counter: u8,
    loss: u8,
    loss_rate: u8,
    rssi: Rc<U16Data<RSSI>>,
    control_input: Rc<SingularData<ControlInput>>,
    notify: Option<Box<dyn Notify>>,
}

#[inline]
fn to_axis(value: u16) -> i32 {
    // [0, 2047] -> [-1024, 1023] -> [-32768, 32736]
    (value as i32).wrapping_sub(0x400) << 5
}

fn scale(data: u16, scale: u8) -> i16 {
    let scaled = to_axis(data) * scale as i32 / 100;
    if scaled > i16::MAX as i32 {
        i16::MAX
    } else if scaled < i16::MIN as i32 {
        i16::MIN
    } else {
        scaled as i16
    }
}

impl SbusReceiver {
    pub fn new() -> Self {
        Self {
            counter: 0,
            loss: 0,
            loss_rate: 0,
            rssi: Rc::new(U16Data::default()),
            control_input: Rc::new(SingularData::default()),
            notify: None,
        }
    }

    pub fn rssi_reader(&self) -> U16DataSource<RSSI> {
        U16DataSource::new(&self.rssi)
    }

    pub fn input_reader(&self) -> SingularDataSource<ControlInput> {
        SingularDataSource::new(&self.control_input)
    }

    pub fn set_notify(&mut self, notify: Box<dyn Notify>) {
        self.notify = Some(notify);
    }

    fn handle_sbus_data(&mut self, data: &SbusData) {
        self.loss += data.frame_lost as u8;
        self.counter += 1;
        if self.counter == 100 {
            self.loss_rate = self.loss;
            self.counter = 0;
        }
        self.rssi.write((100 - self.loss_rate) as RSSI);

        let mut counter = 0;
        let mut input = ControlInput::default();
        for (id, cfg) in config::get().receiver.inputs.0.iter() {
            let channel = cfg.channel as usize;
            if channel > data.channels.len() {
                continue;
            }
            match id {
                InputType::Throttle => input.throttle = scale(data.channels[channel], cfg.scale),
                InputType::Roll => input.roll = scale(data.channels[channel], cfg.scale),
                InputType::Pitch => input.pitch = scale(data.channels[channel], cfg.scale),
                InputType::Yaw => input.yaw = scale(data.channels[channel], cfg.scale),
            }
            counter += 1;
            if counter >= 4 {
                break;
            }
        }
        self.control_input.write(input);
        if let Some(ref mut notify) = self.notify {
            notify.notify()
        }
    }

    pub fn handle(&mut self, ring: &[u8], half: bool) {
        let begin = if half { 0 } else { ring.len() / 2 };
        let end = if half { ring.len() / 2 } else { ring.len() };
        let mut offset = usize::MAX;
        let mut packet = [0u8; 1 + SBUS_PACKET_SIZE];
        for i in begin..end {
            if !is_sbus_packet_end(ring[i]) {
                continue;
            }
            let index = (i + ring.len() - SBUS_PACKET_SIZE) % ring.len();
            if ring[index] == SBUS_PACKET_BEGIN {
                offset = index;
                break;
            }
        }
        if offset == usize::MAX {
            return;
        }
        for i in 0..SBUS_PACKET_SIZE {
            packet[1 + i] = ring[(offset + i) % ring.len()];
        }
        let packet = SbusPacket::from_bytes(&packet).unwrap();
        let sbus_data = packet.parse();
        self.handle_sbus_data(&sbus_data);
    }
}
