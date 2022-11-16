#![cfg_attr(not(test), no_std)]

use arrayvec::ArrayString;
use rp_pico::hal::rtc::{DayOfWeek, DateTime};
use callback::Callback;

pub struct Alarm<C, D, DateFormat>  {
    date: DateFormat,
    description: ArrayString<16>,
    total_duration_sec: u32,
    intense_duration_sec: u32,
    pause_duration_sec: u32,
    callback: C,
    deactivation_callback: D,
    is_active: bool
}

impl<C, D, DateFormat> Alarm<C,D, DateFormat> {
    pub fn new(date: DateFormat, description: ArrayString<16>, total_duration_sec: u32, intense_duration_sec: u32, pause_duration_sec: u32, action: C, deactivation_callback: D) -> Self {
        Self { date, description, total_duration_sec, intense_duration_sec, pause_duration_sec, callback: action, deactivation_callback: deactivation_callback, is_active: true }
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

impl <C:Callback, D:Callback>Triggerable for Alarm <C, D, WeeklyDate>{
    fn trigger(&mut self, current_time: DateTime) -> bool{
        let mut triggered = false;
        if self.is_active && self.is_date_in_activation_period(current_time) {
            self.is_active = self.callback.call();
            if !self.is_active {//If callback has been stopped...
                self.deactivation_callback.call();//...call the deactivation callback.
            }
            triggered = true;
        }
        return triggered
    }
}

impl <C:Callback, D:Callback> Alarm <C, D, WeeklyDate>{
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

        let number_of_seconds_in_a_week = 7 * 24 * 60 * 60;

        let end_time_in_seconds = trigger_time_in_seconds + self.total_duration_sec;

        let is_in_activation_period = if end_time_in_seconds >= trigger_time_in_seconds {
            // Case start < end%second_in_week
            seconds_since_week_start >= trigger_time_in_seconds
                && seconds_since_week_start <= end_time_in_seconds
        } else {
            seconds_since_week_start > trigger_time_in_seconds
                || seconds_since_week_start < end_time_in_seconds % number_of_seconds_in_a_week
        };

        return is_in_activation_period;
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
        fn call(&mut self) -> bool {
            println!("SQUIK :3");
            return true
        }
    }

    #[test]
    fn simple_in_period_date_check() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
            callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 20
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, true);
    }

    #[test]
    fn simple_not_in_period_date_check() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
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
    fn simple_start_of_period() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
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
    fn simple_end_of_period() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 40
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, true);
    }

    #[test]
    fn simple_1sec_after_end_of_period() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 41
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn simple_complicated_case() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
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
    fn simple_complicated_case_false() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 41
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn simple_complicated_case_true_start_limit() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Sunday,
            hour: 23,
            minute: 59,
            second: 59
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, true);
    }

    #[test]
    fn simple_complicated_case_true_end_limit() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 29
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, true);
    }

    #[test]
    fn simple_complicated_case_false_end_limit() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 30
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn simple_complicated_case_false_start_limit() {
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Sunday,
            23,
            59,
            59), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Sunday,
            hour: 23,
            minute: 59,
            second: 58
        };
        let res = alarm.is_date_in_activation_period(time);
        assert_eq!(res, false);
    }

    #[test]
    fn triggr_test_true(){
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                               callback1, callback2);
        let time = DateTime{
            year: 0,
            month: 0,
            day: 0,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 20
        };

        assert_eq!(alarm.trigger(time),true);
    }

    #[test]
    fn triggr_test_false(){
        let callback1 = DummyCallback{};
        let callback2 = DummyCallback{};

        let mut alarm = Alarm::new(WeeklyDate::new(
            DayOfWeek::Monday,
            0,
            0,
            10), ArrayString::<16>::from("descr").unwrap(), 30, 0, 0,
                                   callback1, callback2);
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