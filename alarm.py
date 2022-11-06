import abc
from typing import Callable

class AbstractAlarm:
    def __init__(self,
                 description: str,
                 total_duration_sec: int,
                 intense_duration_sec: int,
                 pause_duration_sec: int
    ):
        self._is_active: bool = True
        self._description: str = description
        self._total_duration_sec: int = total_duration_sec
        self._intense_duration_sec: int = intense_duration_sec
        self._pause_duration_sec: int = pause_duration_sec
        self._callback: Callable = lambda : None
        self._deactivation_callback: Callable = lambda : None

    @abc.abstractmethod
    def trigger(self):
        pass



class WeeklyDate:
    def __init__(self, day_of_week: DayOfWeek, hour: int, minute: int, second: int):
        self._day_of_week: DayOfWeek = day_of_week
        self._hour: int = hour
        self._minute: int = minute
        self._second: int = second

class WeeklyAlarm(AbstractAlarm):
    def __init__(self,
                 description: str,
                 total_duration_sec: int,
                 intense_duration_sec: int,
                 pause_duration_sec: int,
                 weekly_date:
    ):
        super().__init__(description,
                 total_duration_sec,
                 intense_duration_sec,
                 pause_duration_sec)

    def trigger(self, current_time: DateTime):
        if self._is_active and self.is_date_in_activation_period(current_time):
            self._is_active = self._callback.call(components);
        if !self.is_active {// If has been stopped
        self.desactivation_callback.call(components);
        }
        return true;
        }
        return false;

    }


pub trait Triggerable<DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId>{
    fn trigger(&mut self, current_time: DateTime, components: &mut Components<DP,CP, T>) ->bool;
}

impl <DP: PinId + BankPinId, CP: PinId + BankPinId, T: PinId, C:Callback<DP,CP,T>, D:Callback<DP,CP, T>>Triggerable<DP,CP, T> for Alarm <C, D, WeeklyDate> {

}

impl <C, D> Alarm <C, D, WeeklyDate> {
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