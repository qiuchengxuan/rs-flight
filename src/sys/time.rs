use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

#[derive(Copy, Clone, Debug)]
pub enum Error {
    NotImplemented,
    NotInitialized,
}

extern "Rust" {
    fn time_time() -> NaiveTime;
    fn time_date() -> NaiveDate;
    fn time_update(datetime: &NaiveDateTime) -> Result<(), Error>;
}

pub fn date() -> NaiveDate {
    unsafe { time_date() }
}

pub fn time() -> NaiveTime {
    unsafe { time_time() }
}

pub fn now() -> NaiveDateTime {
    NaiveDateTime::new(date(), time())
}

pub fn update(datetime: &NaiveDateTime) -> Result<(), Error> {
    unsafe { time_update(datetime) }
}

#[macro_export]
macro_rules! fake_rtc {
    () => {
        use $crate::jiffies;

        #[no_mangle]
        fn time_date() -> NaiveDate {
            NaiveDate::from_ymd(1970, 1, 1)
        }

        #[no_mangle]
        fn time_time() -> NaiveTime {
            let jiffies = jiffies::get();
            let (seconds, nanos) = (jiffies.as_secs() as u32, jiffies.subsec_nanos());
            NaiveTime::from_num_seconds_from_midnight(seconds, nanos)
        }

        #[no_mangle]
        fn time_update(_datetime: &NaiveDateTime) -> Result<(), Error> {
            Err(Error::NotImplemented)
        }
    };
}
