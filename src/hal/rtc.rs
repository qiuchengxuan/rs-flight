use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};

pub trait Writer {
    fn set_time(&mut self, time: &NaiveTime);
    fn set_date(&mut self, date: &NaiveDate);
    fn set_datetime(&mut self, datetime: &NaiveDateTime);
}

pub trait Reader {
    fn date(&self) -> NaiveDate;
    fn time(&self) -> NaiveTime;
    fn now(&self) -> NaiveDateTime;
}
