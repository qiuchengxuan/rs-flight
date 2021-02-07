use super::serial::Serial;
use alloc::boxed::Box;
use usb_device::bus::{UsbBus, UsbBusAllocator};
use usb_device::prelude::*;
use usbd_serial::{SerialPort, UsbError};

type E = UsbError;

pub struct UsbSerial<B: 'static + UsbBus> {
    pub serial: Serial<E, E, SerialPort<'static, B>>,
    device: UsbDevice<'static, B>,
}

impl<B: 'static + UsbBus> UsbSerial<B> {
    pub fn new(alloc: UsbBusAllocator<B>) -> Self {
        let allocator: &'static mut UsbBusAllocator<B> = Box::leak(Box::new(alloc));
        let serial = Serial(SerialPort::new(allocator));
        let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x16c0, 0x27dd))
            .product("pro-flight")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .build();
        return Self { serial, device };
    }

    pub fn poll(&mut self) -> bool {
        self.device.poll(&mut [&mut self.serial.0])
    }
}
