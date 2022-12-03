use arrayvec::ArrayString;
use core::fmt::Write;
use cortex_m::delay::Delay;
use embedded_hal::digital::v2::InputPin;
use lcd_1602_i2c::{Blink, Lcd};
use rp_pico::{
    hal::{
        gpio::{self, bank0::BankPinId, Function, Input, Pin, PinId, PullUp},
        rtc::{DateTime, DayOfWeek},
        I2C,
    },
    pac::I2C0,
};

pub trait FormatToArrayString {
    fn to_date_arraystring(&self) -> ArrayString<10>;
    fn to_time_arraystring(&self, without_seconds: bool) -> ArrayString<8>;
    fn to_day_of_week_arraystring(&self) -> ArrayString<8>;
}

pub trait FromScreenAndButtons {
    fn from_screen_and_buttons<DP: PinId + BankPinId, CP: PinId + BankPinId, IP: PinId, VP: PinId>(
        lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
        delay: &mut Delay,
        increment_button: &mut Pin<IP, Input<PullUp>>,
        validate_button: &mut Pin<VP, Input<PullUp>>,
    ) -> Self;
}

fn get_day_of_week_string(day_of_week: DayOfWeek) -> ArrayString<8> {
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

#[derive(PartialEq)]
enum ButtonPhase {
    ButtonPhaseDayOfWeek = 0,
    ButtonPhaseTimeHourTens = 1,
    ButtonPhaseTimeHourUnits = 2,
    ButtonPhaseTimeMinuteTens = 3,
    ButtonPhaseTimeMinuteUnits = 4,
    ButtonPhaseFinished = 5,
}

fn get_button_phase_from_u8(button_phase: &u8) -> ButtonPhase {
    return match button_phase {
        0 => ButtonPhase::ButtonPhaseDayOfWeek,
        1 => ButtonPhase::ButtonPhaseTimeHourTens,
        2 => ButtonPhase::ButtonPhaseTimeHourUnits,
        3 => ButtonPhase::ButtonPhaseTimeMinuteTens,
        4 => ButtonPhase::ButtonPhaseTimeMinuteUnits,
        5 => ButtonPhase::ButtonPhaseFinished,
        _ => ButtonPhase::ButtonPhaseFinished,
    };
}

fn get_button_phase_string(button_phase: &ButtonPhase) -> ArrayString<16> {
    let ret = match button_phase {
        ButtonPhase::ButtonPhaseDayOfWeek => "Jour Semaine ?",
        ButtonPhase::ButtonPhaseTimeHourTens => "Dizaine Heure ?",
        ButtonPhase::ButtonPhaseTimeHourUnits => "Unite Heure ?",
        ButtonPhase::ButtonPhaseTimeMinuteTens => "Dizaine Minute ?",
        ButtonPhase::ButtonPhaseTimeMinuteUnits => "Unite Minute ?",
        ButtonPhase::ButtonPhaseFinished => "Finished ?",
    };

    let mut ret_arrstr = ArrayString::<16>::new();
    write!(ret_arrstr, "{: <16}", ret).unwrap();
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
        let mut datetime = DateTime {
            year: 0,
            day: 1,
            month: 1,
            day_of_week: DayOfWeek::Monday,
            hour: 0,
            minute: 0,
            second: 0,
        };

        let mut button_phase = ButtonPhase::ButtonPhaseDayOfWeek;

        loop {
            if increment_button.is_low().unwrap() {
                match button_phase {
                    ButtonPhase::ButtonPhaseDayOfWeek => {
                        datetime.day_of_week =
                            day_of_week_from_u8((datetime.day_of_week as u8 + 1) % 7)
                    }
                    ButtonPhase::ButtonPhaseTimeHourTens => {
                        datetime.hour = (datetime.hour + 10) % 30
                    }
                    ButtonPhase::ButtonPhaseTimeHourUnits => {
                        datetime.hour = ((datetime.hour + 1)
                            % (if datetime.hour / 10 == 2 { 4 } else { 10 }))
                            + datetime.hour / 10 * 10
                    }
                    ButtonPhase::ButtonPhaseTimeMinuteTens => {
                        datetime.minute = (datetime.minute + 10) % 60
                    }
                    ButtonPhase::ButtonPhaseTimeMinuteUnits => {
                        datetime.minute = (datetime.minute + 1) % 10 + datetime.minute / 10 * 10
                    }

                    _ => {}
                }
                while increment_button.is_low().unwrap() {}
                delay.delay_ms(500); // We wait to avoid multiple triggers
            }

            if validate_button.is_low().unwrap() {
                let incr_button_phase = button_phase as u8 + 1;
                button_phase = get_button_phase_from_u8(&incr_button_phase);
                delay.delay_ms(500); // We wait to avoid multiple triggers
                while validate_button.is_low().unwrap() {}
            }

            if button_phase == ButtonPhase::ButtonPhaseFinished {
                lcd.set_blink(Blink::Off).unwrap();
                lcd.clear(delay).unwrap();
                return datetime;
            }
            render(&datetime, lcd, delay, &button_phase);
            // We wait for the next user input.
            while !increment_button.is_low().unwrap() && !validate_button.is_low().unwrap() {}
        }
    }
}

fn render<DP: PinId + BankPinId, CP: PinId + BankPinId>(
    datetime: &DateTime,
    lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
    delay: &mut Delay,
    button_phase: &ButtonPhase,
) {
    let str_lcd_phase = get_button_phase_string(button_phase);
    let str_lcd_value = match button_phase {
        ButtonPhase::ButtonPhaseDayOfWeek => get_day_of_week_string(datetime.day_of_week),
        ButtonPhase::ButtonPhaseTimeHourTens => datetime.to_time_arraystring(true),
        ButtonPhase::ButtonPhaseTimeHourUnits => datetime.to_time_arraystring(true),
        ButtonPhase::ButtonPhaseTimeMinuteTens => datetime.to_time_arraystring(true),
        ButtonPhase::ButtonPhaseTimeMinuteUnits => datetime.to_time_arraystring(true),
        ButtonPhase::ButtonPhaseFinished => ArrayString::<8>::new(),
    };

    lcd.clear(delay).unwrap();

    lcd.set_cursor_position(0, 0).unwrap();
    lcd.write_str(str_lcd_phase.as_str()).unwrap();

    lcd.set_cursor_position(0, 1).unwrap();
    lcd.write_str(str_lcd_value.as_str()).unwrap();

    match button_phase {
        ButtonPhase::ButtonPhaseTimeHourUnits => blink_digit(lcd, 1),
        ButtonPhase::ButtonPhaseTimeHourTens => blink_digit(lcd, 0),
        ButtonPhase::ButtonPhaseTimeMinuteTens => blink_digit(lcd, 3),
        ButtonPhase::ButtonPhaseTimeMinuteUnits => blink_digit(lcd, 4),
        ButtonPhase::ButtonPhaseFinished => {}
        ButtonPhase::ButtonPhaseDayOfWeek => {}
    }
}

fn blink_digit<DP: PinId + BankPinId, CP: PinId + BankPinId>(
    lcd: &mut Lcd<I2C<I2C0, (Pin<DP, Function<gpio::I2C>>, Pin<CP, Function<gpio::I2C>>)>>,
    pos_digit: u8,
) {
    lcd.set_cursor_position(pos_digit, 1).unwrap();
    lcd.set_blink(Blink::On).unwrap();
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

    fn to_time_arraystring(&self, without_seconds: bool) -> ArrayString<8> {
        let mut time_string = ArrayString::<8>::new();
        if without_seconds {
            write!(&mut time_string, "{:0>2}:{:0>2}", self.hour, self.minute).unwrap();
        } else {
            write!(
                &mut time_string,
                "{:0>2}:{:0>2}:{:0>2}",
                self.hour, self.minute, self.second
            )
            .unwrap();
        }
        return time_string;
    }

    fn to_day_of_week_arraystring(&self) -> ArrayString<8> {
        return get_day_of_week_string(self.day_of_week);
    }
}
