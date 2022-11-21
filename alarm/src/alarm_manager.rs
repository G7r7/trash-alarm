use crate::{Armable, Triggerable};
extern crate alloc;
use alloc::vec::Vec;
use rp_pico::hal::rtc::DateTime;
pub struct AlarmManager<T: Triggerable + Armable> {
    alarms: Vec<T>,
}

impl<T: Triggerable + Armable> AlarmManager<T> {
    pub fn new(alarms: Vec<T>) -> Self {
        Self { alarms }
    }

    pub fn rearm_all(&mut self, current_time: &DateTime) {
        for alarm in &mut self.alarms {
            alarm.rearm(current_time);
        }
    }

    pub fn trigger_all(&mut self, current_time: &DateTime) {
        for alarm in &mut self.alarms {
            alarm.trigger(current_time);
        }
    }
}
