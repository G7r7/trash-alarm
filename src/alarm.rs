use arrayvec::ArrayString;
use rp_pico::hal::rtc::DayOfWeek;
use rp_pico::pac::generic::W;
use crate::callbacks::Callback;
use crate::DateTime;
use crate::task::Task;
pub struct Alarm<AA, GA, DateFormat>  {
    date: DateFormat,
    description: ArrayString<16>,
    total_duration_sec: u32,
    intense_duration_sec: u32,
    pause_duration_sec: u32,
    aggressive_action: AA,
    gentle_action: GA,
    is_active: bool
}

pub struct WeeklyDate {
    day_of_week: DayOfWeek,
    hour: u32,
    minute: u32,
    second: u32
}

pub trait Triggerable{
    fn trigger(&self, current_time: DateTime);
}

impl <AA: Callback, GA: Callback>Triggerable for Alarm <AA, GA, WeeklyDate> where AA: Callback, GA: Callback{
    fn trigger(&self, current_time: DateTime) {
        if self.is_active && self.is_date_in_activation_period(current_time) {
            <GA as Callback>::call();
        }
    }
}

impl <AA: Callback, GA: Callback> Alarm <AA, GA, WeeklyDate> where AA: Callback, GA: Callback{
    fn is_date_in_activation_period(&self, current_datetime: DateTime) -> bool {
        let mut seconds_since_start_of_week = 0u32;
        seconds_since_start_of_week += current_datetime.second as u32;
        seconds_since_start_of_week += current_datetime.minute as u32 * 60;
        seconds_since_start_of_week += current_datetime.hour as u32 * 60 * 60;

        let mut start_timestamp_seconds = 0u32;
        start_timestamp_seconds += self.date.second as u32;
        start_timestamp_seconds += self.date.minute as u32 * 60;
        start_timestamp_seconds += self.date.hour as u32 * 60 * 60;
        let time_week_sec = 60 * 60 * 24 * 7;

        let mut end_timestamp_seconds = (start_timestamp_seconds + self.total_duration_sec)  % time_week_sec ;
        return start_timestamp_seconds % time_week_sec < seconds_since_start_of_week % time_week_sec
            && end_timestamp_seconds> seconds_since_start_of_week % time_week_sec;

    }
}
