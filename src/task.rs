use arrayvec::ArrayString;
use rp_pico::hal::rtc::DateTime;

pub struct Task {
    description: ArrayString<16>,
    date: DateTime,
}

impl Task {
    pub fn description(&self) -> ArrayString<16> {
        self.description
    }
    pub fn date(&self) -> DateTime {
        let date = DateTime{
            year: self.date.year,
            month: self.date.month,
            day: self.date.day,
            day_of_week: self.date.day_of_week,
            hour: self.date.hour,
            minute: self.date.minute,
            second: self.date.second
        };
        return date;
    }

    pub fn new(description: ArrayString<16>, date: DateTime) -> Self {
        Self { description, date }
    }
}