use core::mem::MaybeUninit;

use chrono::naive::{NaiveDate, NaiveDateTime, NaiveTime};
use chrono::{Datelike, Timelike};
use drone_cortexm::reg::prelude::*;
use drone_stm32_map::periph::rtc::RtcPeriph;
use drone_stm32_map::reg;

const PREDIV_A: u32 = 0x7F;
const PREDIV_S: u32 = 0x1FFF;

struct BCD(u32);

impl BCD {
    fn tens(&self) -> u32 {
        self.0 / 10
    }

    fn units(&self) -> u32 {
        self.0 % 10
    }
}

impl From<(u32, u32)> for BCD {
    fn from(value: (u32, u32)) -> Self {
        Self((value.0 * 10 + value.1) as u32)
    }
}

impl Into<u8> for BCD {
    fn into(self) -> u8 {
        self.0 as u8
    }
}

#[derive(Copy, Clone)]
pub struct RTCReader {
    tr: reg::rtc::Tr<Crt>,
    dr: reg::rtc::Dr<Crt>,
    ssr: reg::rtc::Ssr<Crt>,
}

impl RTCReader {
    pub fn date(&self) -> NaiveDate {
        let reg = self.dr.load();
        let year: BCD = (reg.yt(), reg.yu()).into();
        let month: BCD = (reg.mt() as u32, reg.mu()).into();
        let day: BCD = (reg.dt(), reg.du()).into();
        NaiveDate::from_ymd(year.0 as i32 + 1970, month.0, day.0)
    }

    pub fn time(&self) -> NaiveTime {
        let reg = self.tr.load();
        let hour: BCD = (reg.ht(), reg.hu()).into();
        let minute: BCD = (reg.mnt(), reg.mnu()).into();
        let second: BCD = (reg.st(), reg.su()).into();
        let sub_second = (PREDIV_S - self.ssr.load().ss()) / (PREDIV_S + 1);
        NaiveTime::from_hms_milli(hour.0, minute.0, second.0, sub_second)
    }
}

unsafe impl Send for RTCReader {}
unsafe impl Sync for RTCReader {}

pub struct RTC {
    tr: reg::rtc::Tr<Crt>,
    dr: reg::rtc::Dr<Crt>,
    wpr: reg::rtc::Wpr<Srt>,
    isr: reg::rtc::Isr<Srt>,
    ssr: reg::rtc::Ssr<Crt>,
}

impl RTC {
    pub fn new(regs: RtcPeriph) -> Self {
        let mut rtc = Self {
            tr: regs.rtc_tr.into_copy(),
            dr: regs.rtc_dr.into_copy(),
            wpr: regs.rtc_wpr,
            isr: regs.rtc_isr,
            ssr: regs.rtc_ssr.into_copy(),
        };
        rtc.disable_write_protect();
        rtc.enter_init();
        regs.rtc_prer.modify(|r| r.write_prediv_s(PREDIV_S)); // 1MHz / 128 / 8192 = 1Hz
        regs.rtc_prer.modify(|r| r.write_prediv_a(PREDIV_A)); // NOTE: two sperate accesses must be performed
        rtc.exit_init();
        rtc.enable_write_protect();
        rtc
    }

    pub fn disable_write_protect(&mut self) {
        self.wpr.store(|r| r.write_key(0xCA));
        self.wpr.store(|r| r.write_key(0x53));
    }

    pub fn enable_write_protect(&mut self) {
        self.wpr.store(|r| r.write_key(0xFF));
    }

    fn enter_init(&mut self) {
        self.isr.modify(|r| r.set_init());
        while !self.isr.initf.read_bit() {}
    }

    fn exit_init(&mut self) {
        self.isr.modify(|r| r.clear_init());
        while self.isr.initf.read_bit() {}
    }

    fn _set_time(&mut self, time: &NaiveTime) {
        let hour = BCD(time.hour());
        let minute = BCD(time.minute());
        let second = BCD(time.second());
        self.tr.modify(|r| {
            r.write_ht(hour.tens()).write_hu(hour.units());
            r.write_mnt(minute.tens()).write_mnu(minute.units());
            r.write_st(second.tens()).write_su(second.units())
        });
    }

    pub fn set_time(&mut self, time: &NaiveTime) {
        self.enter_init();
        self._set_time(time);
        self.exit_init();
    }

    fn _set_date(&mut self, date: &NaiveDate) {
        let year = BCD(core::cmp::max(date.year() as u32, 1970) - 1970);
        let month = BCD(date.month());
        let day = BCD(date.day());
        self.dr.modify(|r| {
            r.write_yt(year.tens()).write_yu(year.units());
            if month.tens() > 0 { r.set_mt() } else { r.clear_mt() }.write_mu(month.units());
            r.write_dt(day.tens()).write_du(day.units());
            r.write_wdu(date.weekday().number_from_monday())
        });
    }

    pub fn set_date(&mut self, date: &NaiveDate) {
        self.enter_init();
        self._set_date(date);
        self.exit_init();
    }

    pub fn set_datetime(&mut self, datetime: &NaiveDateTime) {
        self.enter_init();
        self._set_date(&datetime.date());
        self._set_time(&datetime.time());
        self.exit_init();
    }

    pub fn reader(&self) -> RTCReader {
        RTCReader { tr: self.tr, dr: self.dr, ssr: self.ssr }
    }
}

static mut RTC: Option<RTC> = None;

#[no_mangle]
fn time_update(datetime: &NaiveDateTime) -> Result<(), &'static str> {
    cortex_m::interrupt::free(|_cs| {
        let rtc = match unsafe { RTC.as_mut() } {
            Some(rtc) => rtc,
            None => return Err("RTC not initialized yet"),
        };
        rtc.set_datetime(datetime);
        Ok(())
    })
}

static mut RTC_READER: MaybeUninit<RTCReader> = MaybeUninit::uninit();

#[no_mangle]
fn time_time() -> NaiveTime {
    unsafe { &*RTC_READER.as_ptr() }.time()
}

#[no_mangle]
fn time_date() -> NaiveDate {
    unsafe { &*RTC_READER.as_ptr() }.date()
}

pub fn init(regs: RtcPeriph) {
    let mut rtc = RTC::new(regs);
    rtc.disable_write_protect();
    cortex_m::interrupt::free(|_cs| unsafe {
        RTC_READER = MaybeUninit::new(rtc.reader());
        RTC = Some(rtc);
    })
}
