#![cfg_attr(not(test), no_std)]

use arrayvec::ArrayString;
use rp_pico::hal::rtc::{DayOfWeek, DateTime};
use callback::Callback;

pub struct Alarm<C, DateFormat>  {
    date: DateFormat,
    description: ArrayString<16>,
    action: C,
    is_active: bool
}

impl<C, DateFormat> Alarm<C, DateFormat> {
    pub fn new(date: DateFormat, description: ArrayString<16>, action: C) -> Self {
        Self { date, description, action: action, is_active: true }
    }
}

pub struct WeeklyDate {
    day_of_week: DayOfWeek,
    hour: u32,
    minute: u32,
    second: u32
}

impl WeeklyDate {
    pub fn new(day_of_week: DayOfWeek, hour: u32, minute: u32, second: u32) -> Self {
        Self { day_of_week, hour, minute, second }
    }
}

pub trait Triggerable{
    fn trigger(&mut self, current_time: DateTime) ->bool;
}

impl <C:Callback>Triggerable for Alarm <C, WeeklyDate> where C:Callback{
    fn trigger(&mut self, current_time: DateTime) -> bool{
        if self.is_active && self.is_date_in_activation_period(current_time) {
            self.action.call();
            return true;
        }
        return false;
    }
}

impl <C:Callback> Alarm <C, WeeklyDate>{
    pub fn is_date_in_activation_period(&self, current_datetime: DateTime) -> bool {
        let mut seconds_since_week_start = 0u32;
        seconds_since_week_start += current_datetime.second as u32;
        seconds_since_week_start += current_datetime.minute as u32 * 60;
        seconds_since_week_start += current_datetime.hour as u32 * 60 * 60;
        seconds_since_week_start += current_datetime.day_of_week as u32 * 24 * 60 * 60;

        let mut trigger_time_in_seconds = 0u32;
        trigger_time_in_seconds += self.date.second as u32;
        trigger_time_in_seconds += self.date.minute as u32 * 60;
        trigger_time_in_seconds += self.date.hour as u32 * 60 * 60;
        trigger_time_in_seconds += self.date.day_of_week as u32 * 24 * 60 * 60;

        return seconds_since_week_start == trigger_time_in_seconds
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayString;
    use rp_pico::hal::rtc::DayOfWeek;
    use callback::Callback;
    use crate::{Alarm, DateTime, Triggerable, WeeklyDate};

    struct DummyCallback {}
    impl Callback for DummyCallback {
        fn call(&mut self) {
            println!("SQUIK :3")
        }
    }

    #[test]
    fn simple_date_check_true() {
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 10
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, true);
    }

    #[test]
    fn simple_date_check_false() {
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 0
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn simple_just_before() {
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 9
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn simple_just_after() {
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 0
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn triggr_test_true(){
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 10
        };

        assert_eq!(alarm.trigger(time),true);
    }

    #[test]
    fn triggr_test_false(){
        let callback = DummyCallback{};
        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), callback);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 0
        };

        assert_eq!(alarm.trigger(time),false);
    }
}