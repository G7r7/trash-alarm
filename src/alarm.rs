use arrayvec::ArrayString;
use rp_pico::hal::rtc::DayOfWeek;
use crate::callbacks::Callback;
use crate::DateTime;
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

impl<AA, GA, DateFormat> Alarm<AA, GA, DateFormat> {
    pub fn new(date: DateFormat, description: ArrayString<16>, total_duration_sec: u32, intense_duration_sec: u32, pause_duration_sec: u32, aggressive_action: AA, gentle_action: GA) -> Self {
        Self { date, description, total_duration_sec, intense_duration_sec, pause_duration_sec, aggressive_action, gentle_action, is_active: true }
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

impl <AA: Callback, GA: Callback>Triggerable for Alarm <AA, GA, WeeklyDate> where AA: Callback, GA: Callback{
    fn trigger(&mut self, current_time: DateTime) -> bool{
        if self.is_active && self.is_date_in_activation_period(current_time) {
            self.gentle_action.call();
            return true;
        }
        return false;
    }
}

impl <AA: Callback, GA: Callback> Alarm <AA, GA, WeeklyDate> where AA: Callback, GA: Callback{
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

        let is_in_activation_period = if end_time_in_seconds <= trigger_time_in_seconds {
            // Case start < end%second_in_week
            seconds_since_week_start > trigger_time_in_seconds
                && seconds_since_week_start < end_time_in_seconds
        } else {
            seconds_since_week_start > trigger_time_in_seconds
                || seconds_since_week_start < end_time_in_seconds % number_of_seconds_in_a_week
        };

        return is_in_activation_period;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test1() {
        let res = 1 + 1;
        assert_eq!(res, 2);
    }
}