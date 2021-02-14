use chips::stm32f4::rtc;
use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use pro_flight::hal::rtc::{Reader, Writer};

pub struct RTCWriter(pub rtc::RTC);

impl Writer for RTCWriter {
    fn set_time(&mut self, time: &NaiveTime) {
        self.0.set_time(time)
    }

    fn set_date(&mut self, date: &NaiveDate) {
        self.0.set_date(date)
    }

    fn set_datetime(&mut self, datetime: &NaiveDateTime) {
        self.0.set_datetime(datetime)
    }
}

pub struct RTCReader(pub rtc::RTCReader);

impl Reader for RTCReader {
    fn date(&self) -> NaiveDate {
        self.0.date()
    }

    fn time(&self) -> NaiveTime {
        self.0.time()
    }

    fn now(&self) -> NaiveDateTime {
        NaiveDateTime::new(self.0.date(), self.0.time())
    }
}
