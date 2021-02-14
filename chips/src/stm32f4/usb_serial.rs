use drone_core::log::STDOUT_PORT;
use drone_core::prelude::*;
use stm32f4xx_hal::otg_fs::{UsbBus, USB};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, UsbError};

static mut SERIAL_PORT: Option<SerialPort<'static, UsbBus<USB>>> = None;

fn write_bytes(mut bytes: &[u8]) {
    let serial_port = match unsafe { SERIAL_PORT.as_mut() } {
        Some(port) => port,
        None => return,
    };
    cortex_m::interrupt::free(|_| {
        while bytes.len() > 0 {
            let result = nb::block!({
                match serial_port.write(bytes) {
                    Ok(size) => Ok(size),
                    Err(UsbError::WouldBlock) => Err(nb::Error::WouldBlock),
                    Err(e) => Err(nb::Error::Other(e)),
                }
            });
            match result {
                Ok(size) => bytes = &bytes[size..],
                Err(_) => break,
            }
        }
    });
}

#[no_mangle]
extern "C" fn drone_log_is_enabled(port: u8) -> bool {
    port == STDOUT_PORT && unsafe { SERIAL_PORT.is_some() }
}

#[no_mangle]
extern "C" fn drone_log_write_bytes(_port: u8, buffer: *const u8, count: usize) {
    write_bytes(unsafe { core::slice::from_raw_parts(buffer, count) })
}

#[no_mangle]
extern "C" fn drone_log_write_u8(_port: u8, value: u8) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_write_u16(_port: u8, value: u16) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_write_u32(_port: u8, value: u32) {
    write_bytes(&value.to_be_bytes())
}

#[no_mangle]
extern "C" fn drone_log_flush() {
    if let Some(serial_port) = unsafe { SERIAL_PORT.as_mut() } {
        cortex_m::interrupt::free(|_| serial_port.flush().ok());
    }
}

pub struct UsbPoller(UsbDevice<'static, UsbBus<USB>>);

impl UsbPoller {
    pub fn poll(&mut self, mut receiver: impl FnMut(&[u8])) {
        let mut buf = [0u8; 128];
        let mut size = 0;
        cortex_m::interrupt::free(|_| {
            let serial_port = unsafe { SERIAL_PORT.as_mut() }.unwrap();
            if self.0.poll(&mut [serial_port]) {
                if let Some(sz) = serial_port.read(&mut buf).ok() {
                    size = sz
                }
            }
        });
        if size > 0 {
            receiver(&buf[..size]);
        }
    }
}

type Allocator = UsbBusAllocator<UsbBus<USB>>;

pub fn init(alloc: Allocator) -> UsbPoller {
    let allocator: &'static mut Allocator = Box::leak(Box::new(alloc));
    unsafe { SERIAL_PORT = Some(SerialPort::new(allocator)) }
    let device = UsbDeviceBuilder::new(allocator, UsbVidPid(0x0403, 0x6001))
        .product("pro-flight")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();
    UsbPoller(device)
}
