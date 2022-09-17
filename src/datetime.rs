use arrayvec::ArrayString;
use core::fmt::Write;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::InputPin;
use lcd_1602_i2c::Lcd;
use rp_pico::{
    hal::{
        self,
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::DateTime,
        I2C,
    },
    pac::I2C0,
};

pub trait FormatToArrayString {
    fn to_date_arraystring(&self) -> ArrayString<10>;
    fn to_time_arraystring(&self) -> ArrayString<8>;
}

pub trait FromScreenAndButtons {
    fn from_screen_and_buttons<DP: PinId + BankPinId, CP: PinId + BankPinId, IP: PinId, VP: PinId>(
        lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &mut Delay,
        increment_button: &mut Pin<IP, Input<PullUp>>,
        validate_button: &mut Pin<VP, Input<PullUp>>,
    ) -> Self;
}

impl FromScreenAndButtons for DateTime {
    fn from_screen_and_buttons<
        DP: PinId + BankPinId,
        CP: PinId + BankPinId,
        IP: PinId,
        VP: PinId,
    >(
        lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &mut Delay,
        increment_button: &mut Pin<IP, Input<PullUp>>,
        validate_button: &mut Pin<VP, Input<PullUp>>,
    ) -> Self {
        // Année
        let mut year = 2023u16;
        let mut year_string = ArrayString::<4>::new();
        lcd.clear(delay).unwrap();
        loop {
            if validate_button.is_low().unwrap() {
                break;
            }
            year_string.clear();
            write!(&mut year_string, "{:0>4}", year).unwrap();
            lcd.set_cursor_position(0, 0).unwrap();
            lcd.write_str("Year ?").unwrap();
            lcd.set_cursor_position(0, 1).unwrap();
            lcd.write_str(&year_string.as_str()).unwrap();
            if increment_button.is_low().unwrap() {
                year += 1;
                while increment_button.is_low().unwrap() {}
            }
        }
        return DateTime {
            year: year,
            day: 10,
            month: 09,
            day_of_week: hal::rtc::DayOfWeek::Sunday,
            hour: 23,
            minute: 55,
            second: 0,
        };
    }
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
