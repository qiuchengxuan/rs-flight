use alloc::rc::Rc;
use core::cmp::Ordering;
use core::fmt::{Display, Formatter, Result, Write};

use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::sys::jiffies;

#[derive(Copy, Clone, PartialEq)]
pub enum Level {
    Debug = 0,
    Info,
    Warning,
    Error,
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::Debug => write!(f, "DEBUG"),
            Self::Info => write!(f, "INFO "),
            Self::Warning => write!(f, "WARN "),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

impl PartialOrd for Level {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (*self as usize).partial_cmp(&(*other as usize))
    }
}

static mut LOGGER: Option<OverwritingData<u8>> = None;
static mut LEVEL: Level = Level::Debug;

#[doc(hidden)]
pub fn __write_log(args: core::fmt::Arguments, level: Level) {
    if level < unsafe { LEVEL } {
        return;
    }
    let logger = unsafe { LOGGER.as_mut().unwrap() };
    let jiffies = jiffies::get();
    let seconds = jiffies.as_secs() as u32;
    writeln!(logger, "[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), args).ok();
}

#[doc(hidden)]
pub fn __write_log_literal(message: &'static str, level: Level) {
    if level < unsafe { LEVEL } {
        return;
    }
    let logger = unsafe { LOGGER.as_mut().unwrap() };
    let jiffies = jiffies::get();
    let seconds = jiffies.as_secs() as u32;
    writeln!(logger, "[{:5}.{:03}] {}", seconds, jiffies.subsec_millis(), message).ok();
}

#[doc(hidden)]
#[macro_export]
macro_rules! __format_logger_args {
    ($($args:tt)*) => {
        format_args!($($args)*)
    };
}

#[macro_export]
macro_rules! log {
    ($level:expr, $message:expr) => ({
        let _ = __format_logger_args!($message);
        $crate::components::logger::__write_log_literal($message, $level);
    });
    ($level:expr, $($arg:tt)+) => {
        $crate::components::logger::__write_log(__format_logger_args!($($arg)+), $level);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Debug, $($arg)+);
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Info, $($arg)+);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Warning, $($arg)+);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        log!($crate::components::logger::Level::Error, $($arg)+);
    };
}

pub fn get() -> OverwritingDataSource<u8> {
    let logger = unsafe { &Rc::from_raw(LOGGER.as_ref().unwrap()) };
    core::mem::forget(logger);
    OverwritingDataSource::new(logger)
}

pub fn init(level: Level) {
    unsafe { LOGGER = Some(OverwritingData::new(vec![0u8; 4096])) }
    unsafe { LEVEL = level }
}
