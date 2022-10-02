use crate::callbacks::Callback;
use crate::DateTime;
use crate::task::Task;

pub struct WeekAlarm<AA: Callback, GA: Callback>{
    task: Task,
    aggressive_action: AA,
    gentle_action: GA,
    is_active: bool,
    total_duration_sec: u32,
    intense_duration_sec: u32,
    pause_duration_sec: u32
}

impl <AA: Callback, GA: Callback> WeekAlarm <AA,GA>{
    pub fn trigger(&self, current_time: DateTime){
        if self.is_active && self.is_date_in_activation_period(current_time) {
            <GA as Callback>::call();
        }
    }

    fn is_date_in_activation_period(&self, current_datetime: DateTime) -> bool {
        let mut current_timesptamp_seconds = 0u32;
        current_timesptamp_seconds += current_datetime.second as u32;
        current_timesptamp_seconds += current_datetime.minute as u32 * 60;
        current_timesptamp_seconds += current_datetime.hour as u32 * 60 * 60;

        let start_date = self.task.date();
        let mut start_timestamp_seconds = 0u32;
        start_timestamp_seconds += start_date.second as u32;
        start_timestamp_seconds += start_date.minute as u32 * 60;
        start_timestamp_seconds += start_date.hour as u32 * 60 * 60;
        let time_week = 60 * 60 * 24 * 7;

        let mut end_timestamp_seconds = start_timestamp_seconds + self.total_duration_sec;
        return start_timestamp_seconds % time_week < current_timesptamp_seconds % time_week
           && end_timestamp_seconds % time_week > current_timesptamp_seconds % time_week;

    }
}
