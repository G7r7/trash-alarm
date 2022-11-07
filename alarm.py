import abc
import time
from callbacks import Callback


class AbstractAlarm:
    def __init__(self,
                 description: str,
                 total_duration_sec: int,
                 intense_duration_sec: int,
                 pause_duration_sec: int,
                 callback: Callback,
                 deactivation_callback: Callback
                 ):
        self._is_active: bool = True
        self._description: str = description
        self._total_duration_sec: int = total_duration_sec
        self._intense_duration_sec: int = intense_duration_sec
        self._pause_duration_sec: int = pause_duration_sec
        self._callback: Callback = callback
        self._deactivation_callback = deactivation_callback

    @abc.abstractmethod
    def trigger(self, current_epoch: int) -> bool:
        pass

    @abc.abstractmethod
    def is_date_in_activation_period(self, current_epoch: int) -> bool:
        pass


class WeeklyDate:
    def __init__(self, day_of_week: int, hour: int, minute: int, second: int):
        self.day_of_week: int = day_of_week
        self.hour: int = hour
        self.minute: int = minute
        self.second: int = second


class WeeklyAlarm(AbstractAlarm):
    def __init__(self,
                 description: str,
                 total_duration_sec: int,
                 intense_duration_sec: int,
                 pause_duration_sec: int,
                 callback: Callback,
                 deactivation_callback: Callback,
                 weekly_date: WeeklyDate
                 ):
        super().__init__(description,
                         total_duration_sec,
                         intense_duration_sec,
                         pause_duration_sec,
                         callback,
                         deactivation_callback)
        self._date: WeeklyDate = weekly_date

    def trigger(self, current_epoch: int) -> bool:
        triggered = False
        if self._is_active and self.is_date_in_activation_period(current_epoch):
            self._is_active = self._callback.call()
            if not self._is_active:  # If has been stopped
                self._deactivation_callback.call()
            triggered = True
        return triggered

    def is_date_in_activation_period(self, current_epoch: int) -> bool:
        seconds_since_week_start = 0
        time_tuple = time.gmtime(current_epoch)
        seconds_since_week_start += time_tuple[5]  # seconds
        seconds_since_week_start += time_tuple[4] * 60  # minutes
        seconds_since_week_start += time_tuple[3] * 60 * 60  # hours
        seconds_since_week_start += time_tuple[6] * 24 * 60 * 60  # week day

        trigger_time_in_seconds = 0
        trigger_time_in_seconds += self._date.second
        trigger_time_in_seconds += self._date.minute
        trigger_time_in_seconds += self._date.hour * 60 * 60
        trigger_time_in_seconds += self._date.day_of_week * 24 * 60 * 60

        number_of_seconds_in_a_week = 7 * 24 * 60 * 60

        end_time_in_seconds = trigger_time_in_seconds + self._total_duration_sec

        if end_time_in_seconds >= trigger_time_in_seconds:
            # Case where start < end%second_in_week
            is_in_activation_period = trigger_time_in_seconds <= seconds_since_week_start <= end_time_in_seconds
        else:
            is_in_activation_period = seconds_since_week_start > trigger_time_in_seconds \
                                      or seconds_since_week_start < end_time_in_seconds % number_of_seconds_in_a_week

        return is_in_activation_period
