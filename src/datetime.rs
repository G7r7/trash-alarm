use arrayvec::ArrayString;
use core::fmt::{Error, Write};
use cortex_m::delay::Delay;
use embedded_hal::{blocking::serial::write, digital::v2::InputPin};
use lcd_1602_i2c::Lcd;
use rp_pico::{
    hal::{
        self,
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};

const BUTTON_PHASE_DAY_OF_WEEK: u8 = 0;

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
fn get_time_of_day_string(day_of_week: DayOfWeek) -> ArrayString<8> {
    let ret = match day_of_week {
        DayOfWeek::Monday => "Lundi",
        DayOfWeek::Tuesday => "Mardi",
        DayOfWeek::Wednesday => "Mercredi",
        DayOfWeek::Thursday => "Jeudi",
        DayOfWeek::Friday => "Vendredi",
        DayOfWeek::Saturday => "Samedi",
        DayOfWeek::Sunday => "Dimanche",
    };
    let mut ret_arrstr = ArrayString::<8>::new();
    write!(ret_arrstr, "{: <8}", ret).unwrap();
    // write!(ret_arrstr, "{}", ret).unwrap();
    return ret_arrstr;
}

fn day_of_week_from_u8(v: u8) -> DayOfWeek {
    match v {
        0 => DayOfWeek::Sunday,
        1 => DayOfWeek::Monday,
        2 => DayOfWeek::Tuesday,
        3 => DayOfWeek::Wednesday,
        4 => DayOfWeek::Thursday,
        5 => DayOfWeek::Friday,
        6 => DayOfWeek::Saturday,
        _ => DayOfWeek::Sunday,
    }
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
        let mut day_of_week = DayOfWeek::Monday;
        let mut year_string = ArrayString::<4>::new();
        lcd.clear(delay).unwrap();

        let mut button_phase = BUTTON_PHASE_DAY_OF_WEEK;
        let mut str_button_phase = "Jour semaine ?";
        loop {
            if validate_button.is_low().unwrap() {
                break;
            }
            lcd.set_cursor_position(0, 0).unwrap();
            lcd.write_str(str_button_phase).unwrap();

            if button_phase == BUTTON_PHASE_DAY_OF_WEEK {
                // lcd.set_cursor_position(0, 1).unwrap();
                // lcd.write_str("        ").unwrap();
                lcd.set_cursor_position(0, 1).unwrap();
                lcd.write_str(get_time_of_day_string(day_of_week).as_str())
                    .unwrap();
                if increment_button.is_low().unwrap() {
                    day_of_week = day_of_week_from_u8((day_of_week as u8 + 1) % 7);
                    while increment_button.is_low().unwrap() {}
                }
            }
        }
        return DateTime {
            year: 2022,
            day: 10,
            month: 09,
            day_of_week: day_of_week,
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
