use arrayvec::ArrayString;
use core::fmt::Write;
use rp_pico::hal::rtc::DateTime;

pub trait FormatToArrayString {
    fn to_date_arraystring(&self) -> ArrayString<10>;
    fn to_time_arraystring(&self) -> ArrayString<8>;
}

impl FormatToArrayString for DateTime {
    fn to_date_arraystring(&self) -> ArrayString<10> {
        let mut date_string = ArrayString::<10>::new();
        write!(
            &mut date_string,
            "{:0>4}/{:0>2}/{:0>2}",
            self.year, self.month, self.day
        )
        .unwrap();
        return date_string;
    }

    fn to_time_arraystring(&self) -> ArrayString<8> {
        let mut time_string = ArrayString::<8>::new();
        write!(
            &mut time_string,
            "{:0>2}:{:0>2}:{:0>2}",
            self.hour, self.minute, self.second
        )
        .unwrap();
        return time_string;
    }
}
