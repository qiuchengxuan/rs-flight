use alloc::boxed::Box;
use core::cell::RefCell;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

use super::jiffies;
use crate::hal::rtc::{Reader, Writer};

static mut WRITER: Option<RefCell<Box<dyn Writer>>> = None;
static mut READER: Option<Box<dyn Reader>> = None;

pub fn now() -> NaiveDateTime {
    if let Some(reader) = unsafe { READER.as_ref() } {
        return reader.now();
    }
    let jiffies = jiffies::get();
    let (seconds, nanos) = (jiffies.as_secs() as u32, jiffies.subsec_nanos());
    let time = NaiveTime::from_num_seconds_from_midnight(seconds, nanos);
    NaiveDateTime::new(NaiveDate::from_ymd(1970, 1, 1), time)
}

pub fn update(datetime: &NaiveDateTime) -> Result<(), &'static str> {
    let refcell = match unsafe { WRITER.as_ref() } {
        Some(refcell) => refcell,
        None => return Err("Time module not initialized"),
    };

    let mut writer = match refcell.try_borrow_mut() {
        Ok(w) => w,
        Err(_) => return Err("Time update taking place"),
    };
    writer.set_datetime(datetime);
    Ok(())
}

pub fn init(writer: impl Writer + 'static, reader: impl Reader + 'static) {
    unsafe {
        WRITER = Some(RefCell::new(Box::new(writer)));
        READER = Some(Box::new(reader));
    }
}
